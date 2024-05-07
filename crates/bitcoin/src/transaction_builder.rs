use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use bdk::{
    bitcoin::ScriptBuf,
    database::BatchDatabase,
    wallet::{
        coin_selection::{
            BranchAndBoundCoinSelection, CoinSelectionAlgorithm, LargestFirstCoinSelection, OldestFirstCoinSelection,
        },
        tx_builder::{ChangeSpendPolicy, CreateTx, TxBuilder as BdkTxBuilder},
    },
    FeeRate,
};
use hashbrown::HashSet;
use miniscript::bitcoin::{
    absolute::LockTime, psbt::PartiallySignedTransaction, script::PushBytesBuf, Address, OutPoint,
};
use uuid::Uuid;

use super::account::Account;
use crate::error::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum CoinSelection {
    BranchAndBound,
    LargestFirst,
    OldestFirst,
    Manual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TmpRecipient(pub String, pub String, pub u64);

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
pub struct TxBuilder<D>
where
    D: BatchDatabase,
{
    account: Option<Arc<RwLock<Account<D>>>>,

    pub recipients: Vec<TmpRecipient>,
    pub utxos_to_spend: HashSet<OutPoint>,
    pub change_policy: ChangeSpendPolicy,
    pub fee_rate: Option<FeeRate>,
    pub drain_wallet: bool,
    pub drain_to: Option<ScriptBuf>,
    pub rbf_enabled: bool,
    pub data: Vec<u8>,
    pub coin_selection: CoinSelection,
    pub locktime: Option<LockTime>,
}

impl<D> Clone for TxBuilder<D>
where
    D: BatchDatabase,
{
    fn clone(&self) -> Self {
        TxBuilder {
            account: self.account.clone(),
            recipients: self.recipients.clone(),
            utxos_to_spend: self.utxos_to_spend.clone(),
            change_policy: self.change_policy.clone(),
            fee_rate: self.fee_rate.clone(),
            drain_wallet: self.drain_wallet,
            drain_to: self.drain_to.clone(),
            rbf_enabled: self.rbf_enabled,
            data: self.data.clone(),
            coin_selection: self.coin_selection.clone(),
            locktime: self.locktime.clone(),
        }
    }
}

pub struct ScriptAmount {
    pub script: ScriptBuf,
    pub amount: u64,
}

struct AllocateBalanceAcc {
    remaining: u64,
    recipients: Vec<TmpRecipient>,
}

/// This functions allocates a given balance accross the provided recipients.
/// If recipients total amount is greater than provided balance, the last
/// recipients will be allocated less than initially. "First come, first served"
fn allocate_amount_to_recipients(recipients: Vec<TmpRecipient>, amount: u64) -> Vec<TmpRecipient> {
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
fn correct_recipients_amounts(recipients: Vec<TmpRecipient>, amount_to_remove: u64) -> Vec<TmpRecipient> {
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

impl<D> TxBuilder<D>
where
    D: BatchDatabase,
{
    pub fn new() -> Self {
        TxBuilder {
            account: None,
            recipients: vec![TmpRecipient(Uuid::new_v4().to_string(), String::new(), 0)],
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

    /// Sets the account to be used to finalise the transaction. It is also used
    /// to constrain transaction outputs to not oversize balance
    ///
    /// ```rust, ignore
    /// let tx_builder = TxBuilder::new();
    /// let account = Account::new(master_priv_key, config, ());
    /// # ...
    /// let updated = tx_builder.set_account(account).await.unwrap();
    /// ```
    pub async fn set_account(&self, account: Arc<RwLock<Account<D>>>) -> Result<Self, Error> {
        let balance = &account.read().expect("lock").get_balance()?;

        let tx_builder = TxBuilder {
            account: Some(account.clone()),
            recipients: allocate_amount_to_recipients(self.recipients.clone(), balance.confirmed),
            ..self.clone()
        };

        Ok(tx_builder.constrain_recipient_amounts().await)
    }

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
    pub fn add_recipient(&self) -> Self {
        let mut recipients = self.recipients.clone();
        recipients.append(&mut vec![TmpRecipient(Uuid::new_v4().to_string(), String::new(), 0)]);

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

    async fn constrain_recipient_amounts(&self) -> Self {
        if self.account.is_none() {
            return self.clone();
        } else {
            let result = self.create_pbst_with_coin_selection(true).await;

            match result {
                Err(Error::BdkError(err)) => match err {
                    bdk::Error::InsufficientFunds { needed, available } => {
                        let amount_to_remove = needed - available;

                        TxBuilder {
                            recipients: correct_recipients_amounts(self.recipients.clone(), amount_to_remove),
                            ..self.clone()
                        }
                    }
                    _ => self.clone(),
                },
                _ => self.clone(),
            }
        }
    }

    /// Update either recipient's address or amount at provided index
    ///
    /// # Notes
    ///
    /// If amount is too high (higher than balance-expected fees), it will be
    /// constrained     
    /// ```rust, ignore
    /// let tx_builder = TxBuilder::new();
    /// ...
    /// let updated = tx_builder.update_recipient(1usize, Some("bc1..."), Some(18788.0), Some(BitcoinUnit::SATS)).unwrap();
    /// ```
    pub async fn update_recipient(&self, index: usize, update: (Option<String>, Option<u64>)) -> Self {
        let mut recipients = self.recipients.clone();
        let TmpRecipient(uuid, prev_script, prev_amount) = recipients[index].clone();

        recipients[index] = TmpRecipient(
            uuid,
            update.0.map_or(prev_script, |update| update),
            update.1.map_or(prev_amount, |update| update),
        );

        let tx_builder = TxBuilder {
            recipients,
            ..self.clone()
        };

        tx_builder.constrain_recipient_amounts().await
    }

    /// Update one recipient's amount to max, meaning it sets remaining balance
    /// to him.
    pub async fn update_recipient_amount_to_max(&self, index: usize) -> Result<Self, Error> {
        let mut recipients = self.recipients.clone();
        let TmpRecipient(uuid, script, prev_amount) = recipients[index].clone();

        // account is always in sats so we need to convert it to chosen unit
        let max_amount = match self.account.clone() {
            Some(account) => account.read().unwrap().get_balance()?.confirmed,
            _ => prev_amount,
        };

        recipients[index] = TmpRecipient(uuid, script, max_amount);

        let tx_builder = TxBuilder {
            recipients,
            ..self.clone()
        };

        let updated = tx_builder.constrain_recipient_amounts().await;

        Ok(updated)
    }

    /// Adds an outpoint to the list of outpoints to spend.
    pub fn add_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.insert(utxo_to_spend.clone());

        TxBuilder {
            utxos_to_spend,
            ..self.clone()
        }
    }

    /// Removes an outpoint from the list of outpoints to spend.
    pub fn remove_utxo_to_spend(&self, utxo_to_spend: &OutPoint) -> Self {
        let mut utxos_to_spend = self.utxos_to_spend.clone();
        utxos_to_spend.remove(utxo_to_spend);

        TxBuilder {
            utxos_to_spend,
            ..self.clone()
        }
    }

    /// Empty the list of outpoints to spend
    pub fn clear_utxos_to_spend(&self) -> Self {
        TxBuilder {
            utxos_to_spend: HashSet::new(),
            ..self.clone()
        }
    }

    /// Sets the selected coin selection algorithm
    pub fn set_coin_selection(&self, coin_selection: CoinSelection) -> Self {
        TxBuilder {
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
    pub async fn set_fee_rate(&self, sat_per_vb: f32) -> Self {
        let tx_builder = TxBuilder {
            fee_rate: Some(FeeRate::from_sat_per_vb(sat_per_vb)),
            ..self.clone()
        };

        tx_builder.constrain_recipient_amounts().await
    }

    fn commit_utxos<'a, Cs: CoinSelectionAlgorithm<D>>(
        &self,
        mut tx_builder: BdkTxBuilder<'a, D, Cs, CreateTx>,
    ) -> Result<BdkTxBuilder<'a, D, Cs, CreateTx>, Error> {
        if !self.utxos_to_spend.is_empty() {
            let bdk_utxos: Vec<OutPoint> = self
                .utxos_to_spend
                .iter()
                .map(|utxo| OutPoint::from(utxo.clone()))
                .collect();
            let utxos: &[OutPoint] = &bdk_utxos;
            tx_builder.add_utxos(utxos)?;
        }

        Ok(tx_builder)
    }

    fn create_psbt<'a, Cs: CoinSelectionAlgorithm<D>>(
        &self,
        mut tx_builder: BdkTxBuilder<'a, D, Cs, CreateTx>,
        allow_dust: bool,
    ) -> Result<PartiallySignedTransaction, Error> {
        for TmpRecipient(_uuid, address, amount) in &self.recipients {
            // TODO: here convert amount in sats
            tx_builder.add_recipient(Address::from_str(&address)?.assume_checked().script_pubkey(), *amount);
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

        let (psbt, _) = tx_builder.finish().unwrap();

        Ok(psbt)
    }

    /// Creates a PSBT from current TxBuilder
    ///
    /// The resulting psbt can then be provided to Account.sign() method
    pub async fn create_pbst_with_coin_selection(&self, allow_dust: bool) -> Result<PartiallySignedTransaction, Error> {
        let account = self.account.clone().ok_or(Error::AccountNotFound)?;
        let account_write_lock = account.read().expect("lock");
        let wallet = account_write_lock.get_wallet();

        let updated = match self.coin_selection {
            CoinSelection::BranchAndBound => {
                let tx_builder = wallet.build_tx().coin_selection(BranchAndBoundCoinSelection::default());
                self.create_psbt(tx_builder, allow_dust)
            }
            CoinSelection::LargestFirst => {
                let tx_builder = wallet.build_tx().coin_selection(LargestFirstCoinSelection::default());
                self.create_psbt(tx_builder, allow_dust)
            }
            CoinSelection::OldestFirst => {
                let tx_builder = wallet.build_tx().coin_selection(OldestFirstCoinSelection::default());
                self.create_psbt(tx_builder, allow_dust)
            }
            CoinSelection::Manual => {
                let mut tx_builder = wallet.build_tx().coin_selection(BranchAndBoundCoinSelection::default());

                tx_builder = self.commit_utxos(tx_builder)?;
                self.create_psbt(tx_builder, allow_dust)
            }
        };

        updated
    }
}

#[cfg(test)]
mod tests {
    use bdk::{database::MemoryDatabase, wallet::tx_builder::ChangeSpendPolicy, FeeRate};
    use miniscript::bitcoin::absolute::LockTime;

    use super::{super::transaction_builder::CoinSelection, correct_recipients_amounts, TmpRecipient, TxBuilder};
    use crate::transaction_builder::allocate_amount_to_recipients;

    #[test]
    fn should_allocate_amount_when_no_amount_is_set() {
        let recipients: Vec<TmpRecipient> = vec![TmpRecipient("1".to_string(), "addr1".to_string(), 0)];
        let updated = allocate_amount_to_recipients(recipients, 3400);

        assert_eq!(updated, vec![TmpRecipient("1".to_string(), "addr1".to_string(), 0),]);
    }

    #[test]
    fn should_allocate_amount() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
            TmpRecipient("2".to_string(), "addr2".to_string(), 2100),
            TmpRecipient("3".to_string(), "addr3".to_string(), 3000),
        ];
        let updated = allocate_amount_to_recipients(recipients, 4800);

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
                TmpRecipient("2".to_string(), "addr2".to_string(), 1300),
                TmpRecipient("3".to_string(), "addr3".to_string(), 0)
            ]
        );
    }

    #[test]
    fn should_remove_correct_amount() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
            TmpRecipient("2".to_string(), "addr2".to_string(), 2100),
            TmpRecipient("3".to_string(), "addr3".to_string(), 3000),
        ];
        let updated = correct_recipients_amounts(recipients, 3400);

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
                TmpRecipient("2".to_string(), "addr2".to_string(), 1700),
                TmpRecipient("3".to_string(), "addr3".to_string(), 0)
            ]
        );
    }

    #[test]
    fn should_not_create_negative_amounts() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
            TmpRecipient("2".to_string(), "addr2".to_string(), 2100),
            TmpRecipient("3".to_string(), "addr3".to_string(), 3000),
        ];
        let updated = correct_recipients_amounts(recipients, 9000);

        assert_eq!(
            updated,
            vec![
                TmpRecipient("1".to_string(), "addr1".to_string(), 0),
                TmpRecipient("2".to_string(), "addr2".to_string(), 0),
                TmpRecipient("3".to_string(), "addr3".to_string(), 0)
            ]
        );
    }

    #[test]
    fn should_not_remove_anything() {
        let recipients: Vec<TmpRecipient> = vec![
            TmpRecipient("1".to_string(), "addr1".to_string(), 3500),
            TmpRecipient("2".to_string(), "addr2".to_string(), 2100),
            TmpRecipient("3".to_string(), "addr3".to_string(), 3000),
        ];
        let updated = correct_recipients_amounts(recipients.clone(), 0);

        assert_eq!(updated, recipients);
    }

    #[test]
    fn should_set_enable_rbf() {
        let tx_builder = TxBuilder::<MemoryDatabase>::new();

        let updated = tx_builder.enable_rbf();
        assert_eq!(updated.rbf_enabled, true);

        let updated = tx_builder.disable_rbf();
        assert_eq!(updated.rbf_enabled, false);
    }

    #[test]
    fn should_set_locktime() {
        let tx_builder = TxBuilder::<MemoryDatabase>::new();

        let updated = tx_builder.add_locktime(LockTime::from_consensus(788373));
        assert_eq!(updated.locktime, Some(LockTime::from_consensus(788373)));

        let updated = tx_builder.remove_locktime();
        assert_eq!(updated.locktime, None);
    }

    #[test]
    fn should_set_coin_selection() {
        let tx_builder = TxBuilder::<MemoryDatabase>::new();

        let updated = tx_builder.set_coin_selection(CoinSelection::LargestFirst);
        assert_eq!(updated.coin_selection, CoinSelection::LargestFirst);

        let updated = tx_builder.set_coin_selection(CoinSelection::Manual);
        assert_eq!(updated.coin_selection, CoinSelection::Manual);
    }

    #[test]
    fn should_set_change_policy() {
        let tx_builder = TxBuilder::<MemoryDatabase>::new();

        let updated = tx_builder.set_change_policy(ChangeSpendPolicy::ChangeAllowed);
        assert_eq!(updated.change_policy, ChangeSpendPolicy::ChangeAllowed);

        let updated = tx_builder.set_change_policy(ChangeSpendPolicy::ChangeForbidden);
        assert_eq!(updated.change_policy, ChangeSpendPolicy::ChangeForbidden);
    }

    #[tokio::test]
    async fn should_change_fee_rate() {
        let tx_builder = TxBuilder::<MemoryDatabase>::new();

        let updated = tx_builder.set_fee_rate(15.4).await;
        assert_eq!(updated.fee_rate, Some(FeeRate::from_sat_per_vb(15.4)));
    }

    #[test]
    fn should_add_recipient() {
        let tx_builder = TxBuilder::<MemoryDatabase>::new();

        let updated = tx_builder.add_recipient();
        assert_eq!(updated.recipients.len(), 2);
    }

    #[tokio::test]
    async fn should_update_recipient() {
        let tx_builder = TxBuilder::<MemoryDatabase>::new();

        let updated = tx_builder
            .update_recipient(0, (Some("tb1...xyz".to_string()), Some(15837)))
            .await;

        assert_eq!(updated.recipients[0].1, "tb1...xyz".to_string());
        assert_eq!(updated.recipients[0].2, 15837);

        let updated = tx_builder.update_recipient(0, (None, Some(668932))).await;
        assert_eq!(updated.recipients[0].2, 668932);
    }
}
