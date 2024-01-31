use crate::error::Error;
use bdk::bitcoin::{Transaction, Txid};
use bdk::wallet::{ChangeSet, Update as BdkUpdate};
use bdk::{KeychainKind, Wallet as BdkWallet};
use bdk_chain::{local_chain::Update, ConfirmationTimeHeightAnchor, PersistBackend, TxGraph};
use bdk_esplora::esplora_client::{AsyncClient as AsyncEsploraClient, Builder as EsploraClientBuilder};
use lightning::chain::chaininterface::{BroadcasterInterface, ConfirmationTarget, FeeEstimator};
use lightning::chain::BestBlock;

use std::collections::BTreeMap;
use std::collections::HashMap;

use bdk_esplora::EsploraAsyncExt;

use super::utils::{self, now};

#[derive(Clone)]
struct FeesCache {
    last_update: u64,
    fees_by_block_target: HashMap<String, f64>,
}

/// The minimum feerate we are allowed to send, as specify by LDK.
const MIN_FEERATE: u32 = 253;

// TODO: Stop gap should be a setting
const STOP_GAP: usize = 10;

#[cfg(not(target_arch = "wasm32"))]
const PARALLEL_REQUESTS: usize = 5;
#[cfg(target_arch = "wasm32")]
const PARALLEL_REQUESTS: usize = 1;

/// Blockchain Client, mainly responsible for blockchain syncing and checking for both onchain transactions and lightning channels
///
/// Syncing is a crucial part. BDK propose several to plug into a node, but in order to have PWC compatible with most platform possible, we've chosen for the esplora async http client (actually a wrapper around it, calling our API instead of node directly).
#[derive(Clone)]
pub struct Chain {
    client: AsyncEsploraClient,
    should_poll: bool,
    fees: Option<FeesCache>,
}

impl Chain {
    /// Creates a new blockchain client given a url to an esplora server
    pub fn new(url: Option<String>) -> Result<Self, Error<()>> {
        let url = url.unwrap_or("https://mempool.space/testnet/api".to_string());

        let client = EsploraClientBuilder::new(&url)
            .build_async()
            .map_err(|_| Error::Generic {
                msg: "Could not create client".to_string(),
            })?;

        Ok(Chain {
            client,
            should_poll: false,
            fees: None,
        })
    }

    async fn chain_update<Storage>(
        &self,
        wallet: &BdkWallet<Storage>,
        graph_update: &TxGraph<ConfirmationTimeHeightAnchor>,
    ) -> Result<Update, Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet>,
    {
        let checkpoint = wallet.latest_checkpoint();
        let missing_heights = graph_update.missing_heights(wallet.local_chain());

        let chain_update = self
            .client
            .update_local_chain(checkpoint, missing_heights)
            .await
            .map_err(|_| Error::Generic {
                msg: "Could not update chain locally".to_string(),
            })?;

        Ok(chain_update)
    }

    fn apply_and_commit_update<Storage>(
        wallet: &mut BdkWallet<Storage>,
        update: BdkUpdate,
    ) -> Result<(), Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet>,
    {
        wallet.apply_update(update).map_err(|_| Error::Generic {
            msg: "Couldn't apply wallet sync update".to_string(),
        })?;

        wallet.commit().map_err(|_| Error::Generic {
            msg: "Couldn't commit wallet sync update".to_string(),
        })?;

        println!("Commited");

        Ok(())
    }

    /// Given a stop gap (10 currently, hard-coded) and a descriptor, we query transactions for each script pub key until we reach the stop gap, incrementing address index each time.
    /// After fetching those transactions, we can query the blocks to check their confirmation. We get outpoints to track for spending and we also get unused addresses list
    ///
    /// # Notes
    ///
    /// Full sync at startup and recurrent partial sync should be enough to have the UI always up to date. However, 2 edge cases might happen:
    /// - Transaction received on an already used address: we make the assumption that user only uses Proton Wallet, which prevents addresses reuse so we won't encounter this issue often. We should still offer the possibility to manually trigger a new full sync via a button in the UI.
    /// - Transaction received on an address above stop gap: Stop Gap is hardcoded so far. We should soon offer to change the stop gap setting for a given account, so that he can find transactions sent above the previously defined one.
    pub async fn full_sync<Storage>(
        &self,
        wallet: &BdkWallet<Storage>,
    ) -> Result<
        (
            TxGraph<ConfirmationTimeHeightAnchor>,
            Update,
            BTreeMap<KeychainKind, u32>,
        ),
        Error<Storage>,
    >
    where
        Storage: PersistBackend<ChangeSet>,
    {
        let keychain_spks = wallet.all_unbounded_spk_iters().into_iter().collect::<BTreeMap<_, _>>();

        // The client scans keychain spks for transaction histories, stopping after `stop_gap`
        // is reached. It returns a `TxGraph` update (`graph_update`) and a structure that
        // represents the last active spk derivation indices of keychains
        // (`keychain_indices_update`).
        let (graph_update, last_active_indices) = self
            .client
            .full_scan(keychain_spks, STOP_GAP, PARALLEL_REQUESTS)
            .await
            .map_err(|err| Error::Generic {
                msg: format!("{:?}", err),
            })?;

        let chain_update = self.chain_update(wallet, &graph_update).await?;

        Ok((graph_update, chain_update, last_active_indices))
    }

    /// Partial sync uses already synced transactions, outpoints and unused addresses and tracks them, checking for transaction confirmation, outpoints spending and transactions received on unused addresses
    ///
    /// # Notes
    ///
    /// This has to be done on top of a full sync.
    pub async fn partial_sync<Storage>(
        &self,
        wallet: &BdkWallet<Storage>,
    ) -> Result<(TxGraph<ConfirmationTimeHeightAnchor>, Update), Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet>,
    {
        // Script pubkeys that are not used yet
        let unused_spks = wallet
            .spk_index()
            .unused_spks()
            .map(|(_, _, v)| v.to_owned())
            .collect::<Vec<_>>();

        let chain = wallet.local_chain();
        let chain_tip = chain.tip().block_id();

        // Tx that are yet to be confirmed
        let unconfirmed_txids = wallet
            .tx_graph()
            .list_chain_txs(&*chain, chain_tip)
            .filter(|canonical_tx| !canonical_tx.chain_position.is_confirmed())
            .map(|canonical_tx| canonical_tx.tx_node.txid)
            .collect::<Vec<Txid>>();

        // Tracked utxos
        let utxos = wallet.list_unspent().map(|utxo| utxo.outpoint).collect::<Vec<_>>();

        let graph_update = self
            .client
            .sync(unused_spks, unconfirmed_txids, utxos, PARALLEL_REQUESTS)
            .await
            .map_err(|_| Error::Generic {
                msg: "Could not sync".to_string(),
            })?;

        let chain_update = self.chain_update(wallet, &graph_update).await?;

        Ok((graph_update, chain_update))
    }

    pub fn commit_sync<Storage>(
        &self,
        wallet: &mut BdkWallet<Storage>,
        graph_update: TxGraph<ConfirmationTimeHeightAnchor>,
        chain_update: Update,
        last_active_indices: Option<BTreeMap<KeychainKind, u32>>,
    ) -> Result<(), Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet>,
    {
        let update = BdkUpdate {
            graph: graph_update,
            chain: Some(chain_update),
            last_active_indices: last_active_indices.unwrap_or_default(),
            ..BdkUpdate::default()
        };

        Self::apply_and_commit_update(wallet, update)
    }

    pub fn abort_partial_sync(&mut self) {
        self.should_poll = true;
    }

    pub async fn optionally_update_fees(&mut self) -> Result<(), Error<()>> {
        if self.fees.is_none() || now().as_secs() > self.fees.clone().unwrap().last_update + 60 * 10 {
            let fees_by_block_target = self
                .client
                .get_fee_estimates()
                .await
                .map_err(|_| Error::CannotGetFeeEstimation)?;

            self.fees = Some(FeesCache {
                fees_by_block_target,
                last_update: 0,
            });
        }

        Ok(())
    }

    pub async fn get_best_block(&self) -> Result<BestBlock, Error<()>> {
        let tip_height = self
            .client
            .get_height()
            .await
            .map_err(|_| Error::CannotGetFeeEstimation)?;

        let tip_hash = self
            .client
            .get_block_hash(tip_height)
            .await
            .map_err(|_| Error::CannotGetFeeEstimation)?;

        Ok(BestBlock::new(tip_hash, tip_height))
    }

    // TODO: poll fees
    pub async fn get_fees_estimation(&mut self) -> Result<HashMap<String, f64>, Error<()>> {
        self.optionally_update_fees().await?;

        Ok(self
            .fees
            .clone()
            .expect("Should have fees set at this point")
            .fees_by_block_target)
    }

    pub async fn broadcast(&self, transaction: Transaction) -> Result<(), Error<()>> {
        self.client
            .broadcast(&transaction)
            .await
            .map_err(|e| Error::Generic { msg: e.to_string() })
    }
}

fn default_fee_by_confirmation_target(confirmation_target: ConfirmationTarget) -> u32 {
    match confirmation_target {
        ConfirmationTarget::MinAllowedAnchorChannelRemoteFee => 3 * MIN_FEERATE,
        ConfirmationTarget::MinAllowedNonAnchorChannelRemoteFee => 3 * MIN_FEERATE,
        ConfirmationTarget::ChannelCloseMinimum => 3 * MIN_FEERATE,
        ConfirmationTarget::AnchorChannelFee => 10 * MIN_FEERATE,
        ConfirmationTarget::NonAnchorChannelFee => 20 * MIN_FEERATE,
        ConfirmationTarget::OnChainSweep => 50 * MIN_FEERATE,
    }
}

impl FeeEstimator for Chain {
    fn get_est_sat_per_1000_weight(&self, confirmation_target: ConfirmationTarget) -> u32 {
        let fee_key = match confirmation_target {
            ConfirmationTarget::OnChainSweep => "1",
            ConfirmationTarget::NonAnchorChannelFee => "18",
            ConfirmationTarget::AnchorChannelFee => "144",
            ConfirmationTarget::ChannelCloseMinimum => "144",
            ConfirmationTarget::MinAllowedNonAnchorChannelRemoteFee => "1008",
            ConfirmationTarget::MinAllowedAnchorChannelRemoteFee => "1008",
        };

        let fee = self
            .fees
            .clone()
            .and_then(|fees| fees.fees_by_block_target.get(fee_key).map(|f| *f))
            .unwrap_or(default_fee_by_confirmation_target(confirmation_target) as f64) as u32;

        fee
    }
}

impl BroadcasterInterface for Chain {
    fn broadcast_transactions(&self, txs: &[&Transaction]) {
        let chain_clone = self.clone();
        let txs_clone = txs.iter().map(|tx| (*tx).clone()).collect::<Vec<Transaction>>();

        utils::spawn(async move {
            for tx in txs_clone {
                let cloned_tx = (tx).clone();
                if let Err(_) = chain_clone.broadcast(cloned_tx).await {
                    // log_warn!(logger, "Error broadcasting transaction: {e}")
                }
            }
        });
    }
}
