use std::collections::HashMap;

use andromeda_api::{transaction::ExchangeRateOrTransactionTime, ProtonWalletApiClient};
use andromeda_esplora::{AsyncClient, EsploraAsyncExt};
use async_std::sync::RwLockReadGuard;
use bdk_wallet::{
    bitcoin::{Transaction, Txid},
    chain::spk_client::{FullScanResult, SyncResult},
    KeychainKind, PersistedWallet, WalletPersister,
};

use crate::{account::Account, error::Error, storage::WalletPersisterConnector};

pub const DEFAULT_STOP_GAP: usize = 50;
pub const PARALLEL_REQUESTS: usize = 5;

pub struct BlockchainClient(AsyncClient);

impl BlockchainClient {
    pub fn new(proton_api_client: ProtonWalletApiClient) -> Self {
        let client = AsyncClient::from_client(proton_api_client);
        BlockchainClient(client)
    }

    /// Given a stop gap (10 currently, hard-coded) and a descriptor, we query
    /// transactions for each script pub key until we reach the stop gap,
    /// incrementing address index each time. After fetching those
    /// transactions, we can query the blocks to check their confirmation. We
    /// get outpoints to track for spending and we also get unused addresses
    /// list
    ///
    /// # Notes
    ///
    /// Full sync at startup and recurrent partial sync should be enough to have
    /// the UI always up to date. However, 2 edge cases might happen:
    /// - Transaction received on an already used address: we make the
    ///   assumption that user only uses Proton Wallet, which prevents addresses
    ///   reuse so we won't encounter this issue often. We should still offer
    ///   the possibility to manually trigger a new full sync via a button in
    ///   the UI.
    /// - Transaction received on an address above stop gap: Stop Gap is
    ///   hardcoded so far. We should soon offer to change the stop gap setting
    ///   for a given account, so that he can find transactions sent above the
    ///   previously defined one.
    pub async fn full_sync<'a, C, P>(
        &self,
        account: &Account<C, P>,
        stop_gap: Option<usize>,
    ) -> Result<FullScanResult<KeychainKind>, Error>
    where
        C: WalletPersisterConnector<P>,
        P: WalletPersister,
    {
        let read_lock = account.get_wallet().await;
        let request = read_lock.start_full_scan();

        let update = self.0.full_scan(request, stop_gap.unwrap_or(DEFAULT_STOP_GAP)).await?;

        Ok(update)
    }

    /// Partial sync uses already synced transactions, outpoints and unused
    /// addresses and tracks them, checking for transaction confirmation,
    /// outpoints spending and transactions received on unused addresses
    ///
    /// # Notes
    ///
    /// This has to be done on top of a full sync.
    pub async fn partial_sync<'a, P>(
        &self,
        wallet: RwLockReadGuard<'a, PersistedWallet<P>>,
    ) -> Result<SyncResult, Error>
    where
        P: WalletPersister,
    {
        let chain = wallet.local_chain();
        let chain_tip = chain.tip().block_id();

        let utxos = wallet.list_unspent().map(|utxo| utxo.outpoint).collect::<Vec<_>>();

        let unconfirmed_txids = wallet
            .tx_graph()
            .list_canonical_txs(chain, chain_tip)
            .filter(|canonical_tx| !canonical_tx.chain_position.is_confirmed())
            .map(|canonical_tx| canonical_tx.tx_node.txid)
            .collect::<Vec<Txid>>();

        let request = wallet
            .start_sync_with_revealed_spks()
            .outpoints(utxos.into_iter())
            .txids(unconfirmed_txids.into_iter());

        let update = self.0.sync(request, PARALLEL_REQUESTS).await?;

        Ok(update)
    }

    /// Special minimal sync to check account existence
    pub async fn check_account_existence<'a, P>(
        &self,
        wallet: RwLockReadGuard<'a, PersistedWallet<P>>,
        stop_gap: usize,
    ) -> Result<bool, Error>
    where
        P: WalletPersister,
    {
        let spks = wallet.spk_index().all_unbounded_spk_iters();
        let external_keychain_spks = spks.get(&KeychainKind::External);

        let spks = external_keychain_spks
            .map(|spks| spks.clone().take(stop_gap).collect::<Vec<_>>())
            .unwrap_or_default();

        let results = self.0.many_scripthash_txs(spks).await.ok();

        if let Some(results) = results {
            return Ok(results.values().any(|(_index, txs)| !txs.is_empty()));
        };

        Ok(false)
    }

    /// Returns whether or not the wallet needs to be synced again (new block)
    pub async fn should_sync<'a, P>(&self, wallet: RwLockReadGuard<'a, PersistedWallet<P>>) -> Result<bool, Error>
    where
        P: WalletPersister,
    {
        let tip_hash = self.0.get_tip_hash().await?;
        let latest_chekpoint_hash = wallet.latest_checkpoint().hash();

        Ok(tip_hash != latest_chekpoint_hash)
    }

    /// Returns fee estimations in a Map
    pub async fn get_fees_estimation(&self) -> Result<HashMap<String, f64>, Error> {
        let fees = self.0.get_fee_estimates().await?;

        Ok(fees)
    }

    /// Broadcasts a provided transaction
    pub async fn broadcast(
        &self,
        transaction: Transaction,
        wallet_id: String,
        wallet_account_id: String,
        label: Option<String>,
        exchange_rate_or_transaction_time: ExchangeRateOrTransactionTime,
        address_id: Option<String>,
        body: Option<String>,
        recipients: Option<HashMap<String, String>>,
        is_anonymous: Option<u8>,
    ) -> Result<(), Error> {
        self.0
            .broadcast(
                &transaction,
                wallet_id,
                wallet_account_id,
                label,
                exchange_rate_or_transaction_time,
                address_id,
                body,
                recipients,
                is_anonymous,
            )
            .await?;

        Ok(())
    }
}
