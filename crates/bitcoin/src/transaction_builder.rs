use std::{collections::HashMap, fmt::Debug, str::FromStr};

use bdk_wallet::{
    bitcoin::{absolute::LockTime, script::PushBytesBuf, Address, Amount, FeeRate, OutPoint, ScriptBuf},
    wallet::{
        coin_selection::{
            BranchAndBoundCoinSelection, CoinSelectionAlgorithm, Error as CoinSelectionError,
            LargestFirstCoinSelection, OldestFirstCoinSelection,
        },
        error::CreateTxError,
        tx_builder::{ChangeSpendPolicy, TxBuilder as BdkTxBuilder},
    },
    LocalOutput,
};
use hashbrown::HashSet;
use uuid::Uuid;

use super::account::Account;
use crate::{
    error::Error,
    psbt::{Psbt, TemplatePsbtCoinSelection},
    storage::WalletStore,
};

#[derive(Clone, Debug, PartialEq)]
pub enum CoinSelection {
    BranchAndBound,
    LargestFirst,
    OldestFirst,
    Manual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TmpRecipient(pub String, pub String, pub Amount);

/// BDK's implementation of Transaction builder is quite complete, but we need a
/// struct that enables stateful transaction creation, so we just added a layer
/// on top of it.
///
/// andromeda-bitcoin's Transaction Builder is simply an implementation expose
/// setters and getters to tweak transaction options without mutating it but
/// rather returning the updating version. This implementation fits better with
/// ui-based wallets and can even later be used to provide versioning.
///
/// PWC's implementation support most of BDK's exposed options such as coin
/// selection, RBF (enabled by default), Fee rate selection and many other
///
/// This transaction builder implementation aims at being used to enable both
/// raw transaction building and bitcoin URI processing
/// (bitcoin:tb1....?amount=x&label=y)
#[derive(Debug)]
pub struct TxBuilder<P: WalletStore = ()> {
    /// The account associated with the transaction, if any.
    account: Option<Account<P>>,
    /// The optional partially signed Bitcoin transaction (PSBT) to be used as
    /// template for inputs.
    ///
    /// It should be set every time a PSBT is created from this tx builder, so
    /// that next one use the same input selection. On inputs selection change,
    /// either by coin selection method change or manual utxo selection,
    /// `template_psbt` should be reset to `None` because we expect new coins to
    /// be selected then.
    template_psbt: Option<Psbt>,
    /// A list of recipients for the transaction, including a uuid, their
    /// addresses and the amounts to send.
    pub recipients: Vec<TmpRecipient>,
    /// A set of unspent transaction outputs (UTXOs) that are selected to be
    /// spent in the transaction.
    pub utxos_to_spend: HashSet<OutPoint>,
    /// The policy dictating how change from the transaction should be handled.
    pub change_policy: ChangeSpendPolicy,
    /// The fee rate to be used for the transaction, if specified.
    pub fee_rate: Option<FeeRate>,
    /// A flag indicating whether the entire wallet balance should be drained
    /// into this transaction.
    pub drain_wallet: bool,
    /// An optional script to which any leftover funds should be sent, if
    /// `drain_wallet` is enabled.
    pub drain_to: Option<ScriptBuf>,
    /// A flag indicating whether Replace-By-Fee (RBF) is enabled for this
    /// transaction.
    pub rbf_enabled: bool,
    /// Any additional data to be included in the transaction.
    pub data: Vec<u8>,
    /// The coin selection strategy to use for choosing UTXOs.
    pub coin_selection: CoinSelection,
    /// The locktime (block height or timestamp) at which this transaction can
    /// be included in a block, if specified.
    pub locktime: Option<LockTime>,
}

impl<P: WalletStore> Clone for TxBuilder<P> {
    fn clone(&self) -> Self {
        TxBuilder {
            account: self.account.clone(),
            template_psbt: self.template_psbt.clone(),
            recipients: self.recipients.clone(),
            utxos_to_spend: self.utxos_to_spend.clone(),
            change_policy: self.change_policy,
            fee_rate: self.fee_rate,
            drain_wallet: self.drain_wallet,
            drain_to: self.drain_to.clone(),
            rbf_enabled: self.rbf_enabled,
            data: self.data.clone(),
            coin_selection: self.coin_selection.clone(),
            locktime: self.locktime,
        }
    }
}

pub struct ScriptAmount {
    pub script: ScriptBuf,
    pub amount: u64,
}

struct AllocateBalanceAcc {
    remaining: Amount,
    recipients: Vec<TmpRecipient>,
}

/// This functions allocates a given balance accross the provided recipients.
/// If recipients total amount is greater than provided balance, the last
/// recipients will be allocated less than initially. "First come, first served"
fn allocate_amount_to_recipients(recipients: Vec<TmpRecipient>, amount: Amount) -> Vec<TmpRecipient> {
    let acc_result: AllocateBalanceAcc = recipients.into_iter().fold(
        AllocateBalanceAcc {
            remaining: amount,
            recipients: Vec::new(),
        },
        |acc, current| {
            // If remainingAmount = 123 and recipient's amount is 100, we'll allocate 100
            // If remainingAmount = 73 and recipient's amount is 100, we'll allocate only 73
            let amount_to_allocate = acc.remaining.min(current.2);
            let next_remaining = acc.remaining - amount_to_allocate;

            let mut next_recipients = acc.recipients.clone();

            next_recipients.extend(vec![TmpRecipient(current.0, current.1, amount_to_allocate)]);

            AllocateBalanceAcc {
                remaining: next_remaining,
                recipients: next_recipients,
            }
        },
    );

    acc_result.recipients
}

/// This function remove allocated amount from the last recipient to the first
/// one and returns an array of updated recipients
fn correct_recipients_amounts(recipients: Vec<TmpRecipient>, amount_to_remove: Amount) -> Vec<TmpRecipient> {
    let mut cloned = recipients.clone();
    cloned.reverse(); // R3 R2 R1

    let acc_result: AllocateBalanceAcc = cloned.into_iter().fold(
        AllocateBalanceAcc {
            remaining: amount_to_remove,
            recipients: Vec::new(),
        },
        |acc, current| {
            let amount_to_remove = acc.remaining.min(current.2);

            let new_remaining = acc.remaining - amount_to_remove;
            let new_amount = current.2 - amount_to_remove;

            let mut next_recipients = vec![TmpRecipient(current.0, current.1, new_amount)];

            next_recipients.extend(acc.recipients.clone());

            AllocateBalanceAcc {
                remaining: new_remaining,
                recipients: next_recipients,
            }
        },
    );

    acc_result.recipients
}

impl<P: WalletStore> Default for TxBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P: WalletStore> TxBuilder<P> {
    pub fn new() -> Self {
        TxBuilder {
            account: None,
            template_psbt: None,
            recipients: vec![TmpRecipient(Uuid::new_v4().to_string(), String::new(), Amount::ZERO)],
            utxos_to_spend: HashSet::new(),
            change_policy: ChangeSpendPolicy::ChangeAllowed,
            fee_rate: None,
            drain_wallet: false,
            drain_to: None,
            rbf_enabled: true,
            locktime: None,
            coin_selection: CoinSelection::BranchAndBound,
            data: Vec::new(),
        }
    }

    /// Sets the account to be used to finalise the transaction.
    ///
    /// ```rust, ignore
    /// let tx_builder = TxBuilder::new();
    /// let account = Account::new(master_priv_key, config, ());
    /// # ...
    /// let updated = tx_builder.set_account(account);
    /// ```
    pub fn set_account(&self, account: Account<P>) -> Self {
        TxBuilder {
            account: Some(account.clone()),
            ..self.clone()
        }
    }

    /// Sets the PSBT to use as template for inputs selection
    // pub fn set_template(&mut self, psbt: &Psbt) -> &mut Self {
    //     self.template_psbt = Some(psbt.clone());
    //
    //     self
    // }

    /// Clears internal recipient list
    ///
    /// ```rust, ignore
    /// let tx_builder = TxBuilder::new();
    /// ...
    /// let updated = tx_builder.clear_recipients().unwrap();
    /// ```
    pub fn clear_recipients(&self) -> Self {
        TxBuilder {
            recipients: Vec::new(),
            ..self.clone()
        }
    }

    /// Add a recipient to the internal list
    ///
    /// ```rust, ignore
    /// let tx_builder = TxBuilder::new();
    /// ...
    /// let updated = tx_builder.add_recipient().unwrap();
    /// ```
    pub fn add_recipient(&self, data: Option<(Option<String>, Option<u64>)>) -> Self {
        let mut recipients = self.recipients.clone();

        let address = data.as_ref().and_then(|data| data.0.clone()).unwrap_or_default();

        let amount = data
            .as_ref()
            .and_then(|data| data.1.map(Amount::from_sat))
            .unwrap_or(Amount::ZERO);

        recipients.append(&mut vec![TmpRecipient(Uuid::new_v4().to_string(), address, amount)]);

        TxBuilder {
            recipients,
            ..self.clone()
        }
    }

    /// Remove a recipient from the internal list.
    ///     
    /// ```rust, ignore
    /// let tx_builder = TxBuilder::new();
    /// ...
    /// let updated = tx_builder.remove_recipient(1usize).unwrap();
    /// ```
    pub fn remove_recipient(&self, index: usize) -> Self {
        let mut recipients = self.recipients.clone();

        if index < recipients.len() {
            recipients.remove(index);
        }

        TxBuilder {
            recipients,
            ..self.clone()
        }
    }

    pub async fn constrain_recipient_amounts(&self) -> Self {
        if self.account.is_some() {
            let result = self.create_draft_psbt(true).await;

            if let Err(Error::CreateTx(CreateTxError::CoinSelection(CoinSelectionError::InsufficientFunds {
                needed,
                available,
            }))) = result
            {
                let amount_to_remove = needed - available;

                return TxBuilder {
                    recipients: correct_recipients_amounts(self.recipients.clone(), Amount::from_sat(amount_to_remove)),
                    ..self.clone()
                };
            }
        }

        self.clone()
    }

    /// Update either recipient's address or amount at provided index
    ///  
    /// ```rust, ignore
    /// let tx_builder = TxBuilder::new();
    /// ...
    /// let updated = tx_builder.update_recipient(1usize, Some("bc1..."), Some(18788.0), Some(BitcoinUnit::SATS)).unwrap();
    /// ```
    pub fn update_recipient(&self, index: usize, update: (Option<String>, Option<u64>)) -> Self {
        let mut recipients = self.recipients.clone();
        let TmpRecipient(uuid, prev_script, prev_amount) = recipients[index].clone();

        recipients[index] = TmpRecipient(
            uuid,
            update.0.map_or(prev_script, |update| update),
            update.1.map_or(prev_amount, Amount::from_sat),
        );

        TxBuilder {
            recipients,
            ..self.clone()
        }
    }

    /// Update one recipient's amount to max, meaning it sets remaining balance
    /// to him.
    pub async fn update_recipient_amount_to_max(&self, index: usize) -> Self {
        let mut recipients = self.recipients.clone();
        let TmpRecipient(uuid, script, prev_amount) = recipients[index].clone();

        // account is always in sats so we need to convert it to chosen unit
        let max_amount = match self.account.clone() {
            Some(account) => account.get_balance().await.confirmed,
            _ => prev_amount,
        };

        recipients[index] = TmpRecipient(uuid, script, max_amount);

        TxBuilder {
            recipients,
            ..self.clone()
        }
    }

    /// Adds an outpoint to the list of outpoints to spend.
    pub fn add_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.insert(*utxo_to_spend);

        TxBuilder {
            template_psbt: None,
            utxos_to_spend,
            ..self.clone()
        }
    }

    /// Removes an outpoint from the list of outpoints to spend.
    pub fn remove_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.remove(utxo_to_spend);

        TxBuilder {
            template_psbt: None,
            utxos_to_spend,
            ..self.clone()
        }
    }

    /// Empty the list of outpoints to spend
    pub fn clear_utxos_to_spend(&self) -> Self {
        TxBuilder {
            template_psbt: None,
            utxos_to_spend: HashSet::new(),
            ..self.clone()
        }
    }

    /// Sets the selected coin selection algorithm
    pub fn set_coin_selection(&self, coin_selection: CoinSelection) -> Self {
        TxBuilder {
            template_psbt: None,
            coin_selection,
            ..self.clone()
        }
    }

    /// Enable Replace-By_fee
    pub fn enable_rbf(&self) -> Self {
        TxBuilder {
            rbf_enabled: true,
            ..self.clone()
        }
    }

    /// Disable Replace-By_fee
    pub fn disable_rbf(&self) -> Self {
        TxBuilder {
            rbf_enabled: false,
            ..self.clone()
        }
    }

    /// Adds a locktime to the transaction
    pub fn add_locktime(&self, locktime: LockTime) -> Self {
        TxBuilder {
            locktime: Some(locktime),
            ..self.clone()
        }
    }

    /// Removes the transaction's locktime
    pub fn remove_locktime(&self) -> Self {
        TxBuilder {
            locktime: None,
            ..self.clone()
        }
    }

    /// Do not spend change outputs. This effectively adds all the change
    /// outputs to the "unspendable" list. See TxBuilder.unspendable.
    ///
    /// # Notes
    ///
    /// This can conflict with the list of outpoints to spend
    pub fn set_change_policy(&self, change_policy: ChangeSpendPolicy) -> Self {
        TxBuilder {
            change_policy,
            ..self.clone()
        }
    }

    /// Set a custom fee rate.
    pub fn set_fee_rate(&self, sat_per_vb: u64) -> Self {
        TxBuilder {
            fee_rate: FeeRate::from_sat_per_vb(sat_per_vb),
            ..self.clone()
        }
    }

    fn commit_utxos<'a, Cs: CoinSelectionAlgorithm>(
        &self,
        mut tx_builder: BdkTxBuilder<'a, Cs>,
    ) -> Result<BdkTxBuilder<'a, Cs>, Error> {
        if !self.utxos_to_spend.is_empty() {
            let bdk_utxos: Vec<OutPoint> = self.utxos_to_spend.iter().copied().collect();
            let utxos: &[OutPoint] = &bdk_utxos;
            tx_builder.add_utxos(utxos)?;
        }

        Ok(tx_builder)
    }

    fn finish_tx<Cs: CoinSelectionAlgorithm>(
        &self,
        mut tx_builder: BdkTxBuilder<Cs>,
        allow_dust: bool,
    ) -> Result<Psbt, Error> {
        for TmpRecipient(_uuid, address, amount) in &self.recipients {
            tx_builder.add_recipient(Address::from_str(address)?.assume_checked().script_pubkey(), *amount);
        }

        tx_builder.change_policy(self.change_policy);

        if let Some(fee_rate) = self.fee_rate {
            tx_builder.fee_rate(fee_rate);
        }

        tx_builder.allow_dust(allow_dust);

        if self.drain_wallet {
            tx_builder.drain_wallet();
        }

        if self.rbf_enabled {
            tx_builder.enable_rbf();
        }

        if !&self.data.is_empty() {
            let mut buf = PushBytesBuf::new();
            buf.extend_from_slice(self.data.as_slice())
                .map_err(|_| Error::InvalidData(self.data.clone()))?;

            tx_builder.add_data(&buf.as_push_bytes());
        }

        let psbt = Psbt::new(tx_builder.finish()?);

        // self.set_template(&psbt);

        Ok(psbt)
    }

    /// Creates a PSBT from current TxBuilder
    ///
    /// The resulting psbt can then be provided to Account.sign() method
    pub async fn create_psbt(&self, allow_dust: bool, draft: bool) -> Result<Psbt, Error> {
        let account = self.account.clone().ok_or(Error::AccountNotFound)?;
        let mut write_lock = account.get_mutable_wallet().await;

        let available_utxos = write_lock
            .list_unspent()
            .map(|utxo| {
                let keychain = utxo.keychain;
                (
                    utxo.outpoint.to_string(),
                    (utxo, {
                        write_lock
                            .get_descriptor_for_keychain(keychain)
                            .max_weight_to_satisfy()
                            .unwrap()
                            .to_wu() as usize
                    }),
                )
            })
            .collect::<HashMap<String, (LocalOutput, usize)>>();

        let psbt = {
            if let Some(template_psbt) = self.template_psbt.clone() {
                let tx_builder = write_lock
                    .build_tx()
                    .coin_selection(TemplatePsbtCoinSelection(template_psbt, available_utxos));

                return self.finish_tx(tx_builder, allow_dust);
            }

            let tx_builder = write_lock
                .build_tx()
                .coin_selection(BranchAndBoundCoinSelection::default());

            match self.coin_selection {
                CoinSelection::BranchAndBound => self.finish_tx(tx_builder, allow_dust),
                CoinSelection::LargestFirst => {
                    self.finish_tx(tx_builder.coin_selection(LargestFirstCoinSelection), allow_dust)
                }
                CoinSelection::OldestFirst => {
                    self.finish_tx(tx_builder.coin_selection(OldestFirstCoinSelection), allow_dust)
                }
                CoinSelection::Manual => self.finish_tx(self.commit_utxos(tx_builder)?, allow_dust),
            }
        }?;

        if draft {
            write_lock.cancel_tx(&psbt.extract_tx()?);
        }

        Ok(psbt)
    }

    /// Creates a draft PSBT from current TxBuilder to check if it is valid and
    /// return potential errors. PSBTs returned from this method should not
    /// be broadcasted since indexes are not updated
    pub async fn create_draft_psbt(&self, allow_dust: bool) -> Result<Psbt, Error> {
        let psbt = self.create_psbt(allow_dust, true).await?;
        Ok(psbt)
    }
}

#[cfg(test)]
mod tests {
    use bdk_wallet::{
        bitcoin::{absolute::LockTime, Amount, FeeRate},
        wallet::tx_builder::ChangeSpendPolicy,
    };

    use super::{super::transaction_builder::CoinSelection, correct_recipients_amounts, TmpRecipient, TxBuilder};
    use crate::transaction_builder::allocate_amount_to_recipients;

    #[test]
    fn should_allocate_amount_when_no_amount_is_set() {
        let recipients: Vec<TmpRecipient> = vec![TmpRecipient("1".to_string(), "addr1".to_string(), Amount::ZERO)];
        let updated = allocate_amount_to_recipients(recipients, Amount::from_sat(3400));

        assert_eq!(
            updated,
            vec![TmpRecipient("1".to_string(), "addr1".to_string(), Amount::ZERO),]
        );
    }

    #[test]
    fn should_allocate_amount() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), Amount::from_sat(3500)),
            TmpRecipient("2".to_string(), "addr2".to_string(), Amount::from_sat(2100)),
            TmpRecipient("3".to_string(), "addr3".to_string(), Amount::from_sat(3000)),
        ];
        let updated = allocate_amount_to_recipients(recipients, Amount::from_sat(4800));

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), Amount::from_sat(3500)),
                TmpRecipient("2".to_string(), "addr2".to_string(), Amount::from_sat(1300)),
                TmpRecipient("3".to_string(), "addr3".to_string(), Amount::ZERO)
            ]
        );
    }

    #[test]
    fn should_remove_correct_amount() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), Amount::from_sat(3500)),
            TmpRecipient("2".to_string(), "addr2".to_string(), Amount::from_sat(2100)),
            TmpRecipient("3".to_string(), "addr3".to_string(), Amount::from_sat(3000)),
        ];
        let updated = correct_recipients_amounts(recipients, Amount::from_sat(3400));

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), Amount::from_sat(3500)),
                TmpRecipient("2".to_string(), "addr2".to_string(), Amount::from_sat(1700)),
                TmpRecipient("3".to_string(), "addr3".to_string(), Amount::ZERO)
            ]
        );
    }

    #[test]
    fn should_not_create_negative_amounts() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), Amount::from_sat(3500)),
            TmpRecipient("2".to_string(), "addr2".to_string(), Amount::from_sat(2100)),
            TmpRecipient("3".to_string(), "addr3".to_string(), Amount::from_sat(3000)),
        ];
        let updated = correct_recipients_amounts(recipients, Amount::from_sat(9000));

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), Amount::ZERO),
                TmpRecipient("2".to_string(), "addr2".to_string(), Amount::ZERO),
                TmpRecipient("3".to_string(), "addr3".to_string(), Amount::ZERO)
            ]
        );
    }

    #[test]
    fn should_not_remove_anything() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), Amount::from_sat(3500)),
            TmpRecipient("2".to_string(), "addr2".to_string(), Amount::from_sat(2100)),
            TmpRecipient("3".to_string(), "addr3".to_string(), Amount::from_sat(3000)),
        ];
        let updated = correct_recipients_amounts(recipients.clone(), Amount::ZERO);

        assert_eq!(updated, recipients);
    }

    #[test]
    fn should_set_enable_rbf() {
        let tx_builder = TxBuilder::<()>::new();

        let updated = tx_builder.enable_rbf();
        assert!(updated.rbf_enabled);

        let updated = tx_builder.disable_rbf();
        assert!(!updated.rbf_enabled);
    }

    #[test]
    fn should_set_locktime() {
        let tx_builder = TxBuilder::<()>::new();

        let updated = tx_builder.add_locktime(LockTime::from_consensus(788373));
        assert_eq!(updated.locktime, Some(LockTime::from_consensus(788373)));

        let updated = tx_builder.remove_locktime();
        assert_eq!(updated.locktime, None);
    }

    #[test]
    fn should_set_coin_selection() {
        let tx_builder = TxBuilder::<()>::new();

        let updated = tx_builder.set_coin_selection(CoinSelection::LargestFirst);
        assert_eq!(updated.coin_selection, CoinSelection::LargestFirst);

        let updated = tx_builder.set_coin_selection(CoinSelection::Manual);
        assert_eq!(updated.coin_selection, CoinSelection::Manual);
    }

    #[test]
    fn should_set_change_policy() {
        let tx_builder = TxBuilder::<()>::new();

        let updated = tx_builder.set_change_policy(ChangeSpendPolicy::ChangeAllowed);
        assert_eq!(updated.change_policy, ChangeSpendPolicy::ChangeAllowed);

        let updated = tx_builder.set_change_policy(ChangeSpendPolicy::ChangeForbidden);
        assert_eq!(updated.change_policy, ChangeSpendPolicy::ChangeForbidden);
    }

    #[test]
    fn should_change_fee_rate() {
        let tx_builder = TxBuilder::<()>::new();

        let updated = tx_builder.set_fee_rate(15);
        assert_eq!(updated.fee_rate, FeeRate::from_sat_per_vb(15));
    }

    #[test]
    fn should_add_recipient() {
        let tx_builder = TxBuilder::<()>::new();

        let updated = tx_builder.add_recipient(None);
        assert_eq!(updated.recipients.len(), 2);
    }

    #[tokio::test]
    async fn should_update_recipient() {
        let tx_builder = TxBuilder::<()>::new();

        let updated = tx_builder.update_recipient(0, (Some("tb1...xyz".to_string()), Some(15837)));

        assert_eq!(updated.recipients[0].1, "tb1...xyz".to_string());
        assert_eq!(updated.recipients[0].2, Amount::from_sat(15837));

        let updated = tx_builder.update_recipient(0, (None, Some(668932)));
        assert_eq!(updated.recipients[0].2, Amount::from_sat(668932));
    }
}
