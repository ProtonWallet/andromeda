use std::{fmt::Debug, str::FromStr, sync::Arc};

use bdk_wallet::{
    bitcoin::{absolute::LockTime, script::PushBytesBuf, Address, Amount, FeeRate, OutPoint, ScriptBuf},
    coin_selection::{
        BranchAndBoundCoinSelection, CoinSelectionAlgorithm, InsufficientFunds, LargestFirstCoinSelection,
        OldestFirstCoinSelection, SingleRandomDraw,
    },
    error::CreateTxError,
    tx_builder::{ChangeSpendPolicy, TxBuilder as BdkTxBuilder},
    WalletPersister,
};
use bitcoin::key::rand::RngCore;
use hashbrown::HashSet;
use uuid::Uuid;

use super::account::Account;
use crate::{
    error::Error,
    psbt::Psbt,
    storage::{MemoryPersisted, WalletPersisterConnector},
};

#[derive(Clone, Debug, PartialEq)]
pub enum CoinSelection {
    BranchAndBound,
    LargestFirst,
    OldestFirst,
    Manual,
}

struct FixedRng(pub u32);

impl RngCore for FixedRng {
    fn next_u32(&mut self) -> u32 {
        self.0
    }

    fn next_u64(&mut self) -> u64 {
        self.0 as u64
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let bytes = self.0.to_le_bytes();
            for (i, byte) in chunk.iter_mut().enumerate() {
                *byte = bytes[i];
            }
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
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
pub struct TxBuilder<C: WalletPersisterConnector<P>, P: WalletPersister = MemoryPersisted> {
    /// The account associated with the transaction, if any.
    account: Option<Arc<Account<C, P>>>,
    // A random number set on each tx builder instance to randomize coin selection on BNB fallback algorithm, while
    // keeping deterministic inside the same txbuilder
    random_number: u32,
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

impl<C: WalletPersisterConnector<P>, P: WalletPersister> Clone for TxBuilder<C, P> {
    fn clone(&self) -> Self {
        TxBuilder {
            account: self.account.clone(),
            random_number: self.random_number,
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

impl<C: WalletPersisterConnector<P>, P: WalletPersister> Default for TxBuilder<C, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: WalletPersisterConnector<P>, P: WalletPersister> TxBuilder<C, P> {
    pub fn new() -> Self {
        TxBuilder {
            account: None,
            random_number: bitcoin::key::rand::thread_rng().next_u32(),
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
    pub fn set_account(&self, account: Arc<Account<C, P>>) -> Self
    where
        C: WalletPersisterConnector<P>,
    {
        TxBuilder {
            account: Some(account),
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

            if let Err(Error::CreateTx(CreateTxError::CoinSelection(InsufficientFunds { needed, available }))) = result
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

        if !&self.data.is_empty() {
            let mut buf = PushBytesBuf::new();
            buf.extend_from_slice(self.data.as_slice())
                .map_err(|_| Error::InvalidData(self.data.clone()))?;

            tx_builder.add_data(&buf.as_push_bytes());
        }

        let psbt = Psbt::new(tx_builder.finish_with_aux_rand(&mut FixedRng(self.random_number))?);

        // self.set_template(&psbt);

        Ok(psbt)
    }

    /// Creates a PSBT from current TxBuilder
    ///
    /// The resulting psbt can then be provided to Account.sign() method
    pub async fn create_psbt(&self, allow_dust: bool, draft: bool) -> Result<Psbt, Error> {
        let account = self.account.clone().ok_or(Error::AccountNotFound)?;
        let mut write_lock = account.get_mutable_wallet().await;

        let psbt = {
            let tx_builder = write_lock.build_tx();

            match self.coin_selection {
                CoinSelection::BranchAndBound => self.finish_tx(
                    tx_builder.coin_selection(BranchAndBoundCoinSelection::<SingleRandomDraw>::default()),
                    allow_dust,
                ),
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
    use super::Account;
    use andromeda_common::ScriptType;

    use super::{super::transaction_builder::CoinSelection, correct_recipients_amounts, TmpRecipient, TxBuilder};

    use std::{str::FromStr, sync::Arc};

    use andromeda_api::{tests::utils::setup_test_connection, BASE_WALLET_API_V1};
    use andromeda_common::Network;
    use bdk_wallet::{
        bitcoin::{
            absolute::LockTime,
            bip32::{DerivationPath, Xpriv},
            Amount, FeeRate, NetworkKind,
        },
        tx_builder::ChangeSpendPolicy,
    };
    use wiremock::{
        matchers::{body_string_contains, method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{blockchain_client::BlockchainClient, mnemonic::Mnemonic, read_mock_file, storage::MemoryPersisted};

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
        let tx_builder = TxBuilder::<MemoryPersisted>::new();

        let updated = tx_builder.enable_rbf();
        assert!(updated.rbf_enabled);

        let updated = tx_builder.disable_rbf();
        assert!(!updated.rbf_enabled);
    }

    #[test]
    fn should_set_locktime() {
        let tx_builder = TxBuilder::<MemoryPersisted>::new();

        let updated = tx_builder.add_locktime(LockTime::from_consensus(788373));
        assert_eq!(updated.locktime, Some(LockTime::from_consensus(788373)));

        let updated = tx_builder.remove_locktime();
        assert_eq!(updated.locktime, None);
    }

    #[test]
    fn should_set_coin_selection() {
        let tx_builder = TxBuilder::<MemoryPersisted>::new();

        let updated = tx_builder.set_coin_selection(CoinSelection::LargestFirst);
        assert_eq!(updated.coin_selection, CoinSelection::LargestFirst);

        let updated = tx_builder.set_coin_selection(CoinSelection::Manual);
        assert_eq!(updated.coin_selection, CoinSelection::Manual);
    }

    #[test]
    fn should_set_change_policy() {
        let tx_builder = TxBuilder::<MemoryPersisted>::new();

        let updated = tx_builder.set_change_policy(ChangeSpendPolicy::ChangeAllowed);
        assert_eq!(updated.change_policy, ChangeSpendPolicy::ChangeAllowed);

        let updated = tx_builder.set_change_policy(ChangeSpendPolicy::ChangeForbidden);
        assert_eq!(updated.change_policy, ChangeSpendPolicy::ChangeForbidden);
    }

    #[test]
    fn should_change_fee_rate() {
        let tx_builder = TxBuilder::<MemoryPersisted>::new();

        let updated = tx_builder.set_fee_rate(15);
        assert_eq!(updated.fee_rate, FeeRate::from_sat_per_vb(15));
    }

    #[test]
    fn should_add_recipient() {
        let tx_builder = TxBuilder::<MemoryPersisted>::new();

        let updated = tx_builder.add_recipient(None);
        assert_eq!(updated.recipients.len(), 2);
    }

    #[test]
    fn test_remove_recipient() {
        let mut tx_builder = TxBuilder::<MemoryPersisted>::new();

        tx_builder = tx_builder.add_recipient(None);
        assert_eq!(tx_builder.recipients.len(), 2);

        tx_builder = tx_builder.remove_recipient(0);
        assert_eq!(tx_builder.recipients.len(), 1);
    }

    #[tokio::test]
    async fn should_update_recipient() {
        let tx_builder = TxBuilder::<MemoryPersisted>::new();

        let updated = tx_builder.update_recipient(0, (Some("tb1...xyz".to_string()), Some(15837)));

        assert_eq!(updated.recipients[0].1, "tb1...xyz".to_string());
        assert_eq!(updated.recipients[0].2, Amount::from_sat(15837));

        let updated = tx_builder.update_recipient(0, (None, Some(668932)));
        assert_eq!(updated.recipients[0].2, Amount::from_sat(668932));
    }

    fn set_test_account_regtest(
        script_type: ScriptType,
        derivation_path: &str,
    ) -> Account<MemoryPersisted, MemoryPersisted> {
        let network = NetworkKind::Test;
        let mnemonic = Mnemonic::from_string(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower".to_string(),
        )
        .unwrap();
        let master_secret_key = Xpriv::new_master(network, &mnemonic.inner().to_seed("")).unwrap();

        let derivation_path = DerivationPath::from_str(derivation_path).unwrap();

        Account::new(
            master_secret_key,
            Network::Regtest,
            script_type,
            derivation_path,
            MemoryPersisted {},
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_build_transaction_flow() {
        let mut tx_builder = TxBuilder::<MemoryPersisted>::new();

        // create account and do full sync, balance will be 8781
        let account = set_test_account_regtest(ScriptType::NativeSegwit, "m/84'/1'/0'");

        let mock_server = MockServer::start().await;

        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);

        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "b6c3616a787f87ed96b70770d84d45acf637ed3ad6f2706b2dfc282cc3ba4c05",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5eac955f250ff14fd8c61e29e9531bc3e49d69038981a1344e88b985bd200a29",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        // do full sync
        let update = client.full_sync(&account, None).await.unwrap();
        account
            .apply_update(update)
            .await
            .map_err(|_e| "ERROR: could not apply sync update")
            .unwrap();

        // set account
        tx_builder = tx_builder.set_account(Arc::new(account));

        // test add recipient
        tx_builder = tx_builder.add_recipient(Some((
            Some("bcrt1qh3nltpdyugldpz2hc294k9jwyy9s3953yg7g9j".to_string()),
            None,
        )));
        assert_eq!(tx_builder.recipients.len(), 2);
        assert_eq!(tx_builder.recipients[0].1, "");
        assert_eq!(tx_builder.recipients[0].2.to_sat(), 0);
        assert_eq!(
            tx_builder.recipients[1].1,
            "bcrt1qh3nltpdyugldpz2hc294k9jwyy9s3953yg7g9j"
        );
        assert_eq!(tx_builder.recipients[1].2.to_sat(), 0);

        // test update to max amount
        tx_builder = tx_builder.update_recipient_amount_to_max(1).await;
        assert_eq!(
            tx_builder.recipients[1].1,
            "bcrt1qh3nltpdyugldpz2hc294k9jwyy9s3953yg7g9j"
        );
        assert_eq!(tx_builder.recipients[1].2.to_sat(), 8781);

        // test update recipient
        tx_builder = tx_builder.update_recipient(
            0,
            (
                Some("bcrt1qekjrshcthdqafs0du85llvkwhg25zzpc8ztj4h".to_string()),
                Some(2333),
            ),
        );
        tx_builder = tx_builder.update_recipient(
            1,
            (
                Some("bcrt1qh3nltpdyugldpz2hc294k9jwyy9s3953yg7g9j".to_string()),
                Some(1234),
            ),
        );
        assert_eq!(tx_builder.recipients.len(), 2);
        assert_eq!(
            tx_builder.recipients[0].1,
            "bcrt1qekjrshcthdqafs0du85llvkwhg25zzpc8ztj4h"
        );
        assert_eq!(tx_builder.recipients[0].2.to_sat(), 2333);
        assert_eq!(
            tx_builder.recipients[1].1,
            "bcrt1qh3nltpdyugldpz2hc294k9jwyy9s3953yg7g9j"
        );
        assert_eq!(tx_builder.recipients[1].2.to_sat(), 1234);

        // test constrain recipient amounts
        tx_builder.constrain_recipient_amounts().await;
        assert_eq!(tx_builder.recipients.len(), 2);
        assert_eq!(
            tx_builder.recipients[0].1,
            "bcrt1qekjrshcthdqafs0du85llvkwhg25zzpc8ztj4h"
        );
        assert_eq!(tx_builder.recipients[0].2.to_sat(), 2333);
        assert_eq!(
            tx_builder.recipients[1].1,
            "bcrt1qh3nltpdyugldpz2hc294k9jwyy9s3953yg7g9j"
        );
        assert_eq!(tx_builder.recipients[1].2.to_sat(), 1234);

        // test set coin selection
        tx_builder = tx_builder.set_coin_selection(CoinSelection::LargestFirst);
        assert_eq!(tx_builder.coin_selection, CoinSelection::LargestFirst);

        // test rbf enable/disable
        tx_builder = tx_builder.disable_rbf();
        assert_eq!(tx_builder.rbf_enabled, false);
        tx_builder = tx_builder.enable_rbf();
        assert_eq!(tx_builder.rbf_enabled, true);

        // test set/unset locktime
        let seconds: u32 = 1653195600; // May 22nd, 5am UTC.
        let lock_time = LockTime::from_time(seconds).expect("valid time");
        tx_builder = tx_builder.add_locktime(lock_time);
        assert_eq!(tx_builder.locktime.unwrap(), lock_time);
        tx_builder = tx_builder.remove_locktime();
        assert!(tx_builder.locktime.is_none());

        // test set change policy
        tx_builder = tx_builder.set_change_policy(ChangeSpendPolicy::OnlyChange);
        assert_eq!(tx_builder.change_policy, ChangeSpendPolicy::OnlyChange);

        // test set fee rate
        tx_builder = tx_builder.set_fee_rate(399);
        assert_eq!(tx_builder.fee_rate.unwrap().to_sat_per_vb_floor(), 399);

        // test create psbt
        let psbt = tx_builder.create_psbt(true, false).await;
        // InsufficientFunds error
        assert!(psbt.is_err());

        // test create draft psbt
        let psbt = tx_builder.create_draft_psbt(false).await;
        // InsufficientFunds error
        assert!(psbt.is_err());
    }
}
