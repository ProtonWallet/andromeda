use crate::error::Error;

use bdk::wallet::{ChangeSet, Update as BdkUpdate};
use bdk::Wallet as BdkWallet;
use bdk_chain::local_chain::Update;
use bdk_chain::{ConfirmationTimeAnchor, PersistBackend, TxGraph};
use bdk_esplora::esplora_client::AsyncClient as AsyncEsploraClient;
use miniscript::bitcoin::{Transaction, Txid};

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::thread::sleep;
use std::time::Duration;

use bdk_esplora::EsploraAsyncExt;

// TODO: Stop gap should be a setting
const STOP_GAP: usize = 10;
const PARALLEL_REQUESTS: usize = 5;
pub struct Chain {
    client: AsyncEsploraClient,
    should_poll: bool,
}

impl Chain {
    pub fn new(client: AsyncEsploraClient) -> Self {
        Chain {
            client,
            should_poll: false,
        }
    }

    async fn chain_update<Storage>(
        &self,
        wallet: &mut BdkWallet<Storage>,
        graph_update: &TxGraph<ConfirmationTimeAnchor>,
    ) -> Result<Update, Error>
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

    fn apply_and_commit_update<Storage>(wallet: &mut BdkWallet<Storage>, update: BdkUpdate) -> Result<(), Error>
    where
        Storage: PersistBackend<ChangeSet>,
    {
        wallet.apply_update(update).map_err(|_| Error::Generic {
            msg: "Couldn't apply wallet sync update".to_string(),
        })?;

        wallet.commit().map_err(|_| Error::Generic {
            msg: "Couldn't commit wallet sync update".to_string(),
        })?;

        Ok(())
    }

    pub async fn full_sync<Storage>(&self, wallet: &mut BdkWallet<Storage>) -> Result<(), Error>
    where
        Storage: PersistBackend<ChangeSet>,
    {
        let keychain_spks = wallet.spks_of_all_keychains().into_iter().collect::<BTreeMap<_, _>>();

        // The client scans keychain spks for transaction histories, stopping after `stop_gap`
        // is reached. It returns a `TxGraph` update (`graph_update`) and a structure that
        // represents the last active spk derivation indices of keychains
        // (`keychain_indices_update`).
        let (graph_update, last_active_indices) = self
            .client
            .scan_txs_with_keychains(
                keychain_spks,
                core::iter::empty(),
                core::iter::empty(),
                STOP_GAP,
                PARALLEL_REQUESTS,
            )
            .await
            .map_err(|_| Error::Generic {
                msg: "Could not scan".to_string(),
            })?;

        let chain_update = self.chain_update(wallet, &graph_update).await?;

        let update = BdkUpdate {
            last_active_indices,
            graph: graph_update,
            chain: Some(chain_update),
        };

        Self::apply_and_commit_update(wallet, update)
    }

    pub async fn partial_sync<Storage>(&self, wallet: &mut BdkWallet<Storage>) -> Result<(), Error>
    where
        Storage: PersistBackend<ChangeSet>,
    {
        // Script pubkeys that are not used yet
        let unused_spks = wallet
            .spk_index()
            .unused_spks(..)
            .map(|(_, v)| v.to_owned())
            .collect::<Vec<_>>();

        let chain = wallet.local_chain();
        let chain_tip = chain.tip().map(|cp| cp.block_id()).unwrap_or_default();
        let init_outpoints = wallet.spk_index().outpoints().iter().cloned();
        // Tx that are yet to be confirmed
        let unconfirmed_txids = wallet
            .tx_graph()
            .list_chain_txs(&*chain, chain_tip)
            .filter(|canonical_tx| !canonical_tx.chain_position.is_confirmed())
            .map(|canonical_tx| canonical_tx.tx_node.txid)
            .collect::<Vec<Txid>>();

        // Tracked utxos
        let utxos = wallet
            .tx_graph()
            .filter_chain_unspents(&*chain, chain_tip, init_outpoints)
            .map(|(_, utxo)| utxo.outpoint)
            .collect::<Vec<_>>();

        let graph_update = self
            .client
            .scan_txs(unused_spks, unconfirmed_txids, utxos, PARALLEL_REQUESTS)
            .await
            .map_err(|_| Error::Generic {
                msg: "Could not sync".to_string(),
            })?;

        let chain_update = self.chain_update(wallet, &graph_update).await?;

        let update = BdkUpdate {
            graph: graph_update,
            chain: Some(chain_update),
            ..BdkUpdate::default()
        };

        Self::apply_and_commit_update(wallet, update)
    }

    pub async fn poll_partial_sync<Storage>(&mut self, wallet: &mut BdkWallet<Storage>, poll_interval: Option<u64>)
    where
        Storage: PersistBackend<ChangeSet>,
    {
        self.should_poll = true;

        while self.should_poll {
            let _ = self.partial_sync(wallet).await;

            sleep(Duration::from_secs(poll_interval.unwrap_or(60u64)))
        }
    }

    pub fn abort_partial_sync(&mut self) {
        self.should_poll = true;
    }

    pub async fn get_fees_estimation(&self) -> Result<HashMap<String, f64>, Error> {
        let fees = self
            .client
            .get_fee_estimates()
            .await
            .map_err(|_| Error::CannotGetFeeEstimation)?;

        Ok(fees)
    }

    pub async fn broadcast(&self, transaction: Transaction) -> Result<(), Error> {
        self.client
            .broadcast(&transaction)
            .await
            .map_err(|e| Error::Generic { msg: e.to_string() })
    }
}
