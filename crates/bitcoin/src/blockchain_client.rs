use std::collections::HashMap;

use andromeda_api::{transaction::ExchangeRateOrTransactionTime, ProtonWalletApiClient};
use andromeda_common::utils::now;
use andromeda_esplora::{AsyncClient, EsploraAsyncExt};
use async_std::sync::MutexGuard;
use bdk_wallet::{
    chain::spk_client::{FullScanResult, SyncRequest, SyncResult},
    wallet::Wallet as BdkWallet,
    KeychainKind,
};
use bitcoin::{Script, Transaction, Txid};

use crate::error::Error;

pub const DEFAULT_STOP_GAP: usize = 30;
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
    pub async fn full_sync<'a>(
        &self,
        wallet: MutexGuard<'a, BdkWallet>,
        stop_gap: Option<usize>,
    ) -> Result<FullScanResult<KeychainKind>, Error> {
        fn generate_inspect(kind: KeychainKind) -> impl FnMut(u32, &Script) + Send + Sync + 'static {
            let mut once = Some(());
            move |spk_i, _| {
                match once.take() {
                    Some(_) => print!("\nScanning keychain [{:?}]", kind),
                    None => print!(" {:<3}", spk_i),
                };
            }
        }

        let request = wallet
            .start_full_scan()
            .inspect_spks_for_keychain(KeychainKind::External, generate_inspect(KeychainKind::External))
            .inspect_spks_for_keychain(KeychainKind::Internal, generate_inspect(KeychainKind::Internal));

        let mut update = self
            .0
            .full_scan(request, stop_gap.unwrap_or(DEFAULT_STOP_GAP), PARALLEL_REQUESTS)
            .await?;
        let _ = update.graph_update.update_last_seen_unconfirmed(now().as_secs());

        Ok(update)
    }

    pub fn commit_scan(
        &self,
        wallet: &mut MutexGuard<'_, BdkWallet>,
        update: FullScanResult<KeychainKind>,
    ) -> Result<(), Error> {
        wallet.apply_update(update)?;
        wallet.commit()?;

        Ok(())
    }

    /// Partial sync uses already synced transactions, outpoints and unused
    /// addresses and tracks them, checking for transaction confirmation,
    /// outpoints spending and transactions received on unused addresses
    ///
    /// # Notes
    ///
    /// This has to be done on top of a full sync.
    pub async fn partial_sync<'a>(&self, wallet: MutexGuard<'a, BdkWallet>) -> Result<SyncResult, Error> {
        let cp = wallet.latest_checkpoint();

        let chain = wallet.local_chain();
        let chain_tip = chain.tip().block_id();

        let mut request = SyncRequest::from_chain_tip(cp.clone());

        // Script pubkeys that are not used yet
        let unused_spks = wallet
            .spk_index()
            .unused_spks()
            .map(|(_k, _i, spk)| spk.to_owned())
            .collect::<Vec<_>>();

        let utxos = wallet.list_unspent().map(|utxo| utxo.outpoint).collect::<Vec<_>>();

        let unconfirmed_txids = wallet
            .tx_graph()
            .list_chain_txs(chain, chain_tip)
            .filter(|canonical_tx| !canonical_tx.chain_position.is_confirmed())
            .map(|canonical_tx| canonical_tx.tx_node.txid)
            .collect::<Vec<Txid>>();

        request = request
            .chain_spks(unused_spks.into_iter())
            .chain_outpoints(utxos.into_iter())
            .chain_txids(unconfirmed_txids.into_iter());

        let total_spks = request.spks.len();
        let total_txids = request.txids.len();
        let total_ops = request.outpoints.len();

        request = request
            .inspect_spks({
                let mut visited = 0;
                move |_| {
                    visited += 1;
                    eprintln!(" [ {:>6.2}% ]", (visited * 100) as f32 / total_spks as f32)
                }
            })
            .inspect_txids({
                let mut visited = 0;
                move |_| {
                    visited += 1;
                    eprintln!(" [ {:>6.2}% ]", (visited * 100) as f32 / total_txids as f32)
                }
            })
            .inspect_outpoints({
                let mut visited = 0;
                move |_| {
                    visited += 1;
                    eprintln!(" [ {:>6.2}% ]", (visited * 100) as f32 / total_ops as f32)
                }
            });

        let mut update = self.0.sync(request, PARALLEL_REQUESTS).await?;

        // Update last seen unconfirmed
        let _ = update.graph_update.update_last_seen_unconfirmed(now().as_secs());

        Ok(update)
    }

    pub fn commit_sync(&self, wallet: &mut MutexGuard<'_, BdkWallet>, update: SyncResult) -> Result<(), Error> {
        wallet.apply_update(update)?;
        wallet.commit()?;

        Ok(())
    }

    /// Returns whether or not the wallet needs to be synced again (new block)
    pub async fn should_sync<'a>(&self, wallet: MutexGuard<'a, BdkWallet>) -> Result<bool, Error> {
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
        subject: Option<String>,
        body: Option<String>,
    ) -> Result<(), Error> {
        self.0
            .broadcast(
                &transaction,
                wallet_id,
                wallet_account_id,
                label,
                exchange_rate_or_transaction_time,
                address_id,
                subject,
                body,
            )
            .await?;

        Ok(())
    }
}
