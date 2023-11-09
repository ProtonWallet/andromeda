use crate::wallet::{Update, Wallet};

use bdk::wallet::Update as BdkUpdate;
use bdk::Error as BdkError;
use bdk_esplora::esplora_client::{BlockingClient, Builder};
use bdk_esplora::EsploraExt;

use std::sync::Arc;

pub struct EsploraClient(BlockingClient);

impl EsploraClient {
    pub fn new(url: String) -> Self {
        let client = Builder::new(url.as_str()).build_blocking().unwrap();
        Self(client)
    }

    // This is a temporary solution for scanning. The long-term solution involves not passing
    // the wallet to the client at all.
    pub fn scan(
        &self,
        wallet: Arc<Wallet>,
        stop_gap: u64,
        parallel_requests: u64,
    ) -> Result<Arc<Update>, BdkError> {
        let wallet = wallet.get_wallet();

        let previous_tip = wallet.latest_checkpoint();
        let keychain_spks = wallet.spks_of_all_keychains().into_iter().collect();

        let (update_graph, last_active_indices) = self
            .0
            .scan_txs_with_keychains(
                keychain_spks,
                None,
                None,
                stop_gap as usize,
                parallel_requests as usize,
            )
            .unwrap();

        let missing_heights = update_graph.missing_heights(wallet.local_chain());
        let chain_update = self
            .0
            .update_local_chain(previous_tip, missing_heights)
            .unwrap();

        let update = BdkUpdate {
            last_active_indices,
            graph: update_graph,
            chain: Some(chain_update),
        };

        Ok(Arc::new(Update(update)))
    }

    // pub fn sync();

    // pub fn broadcast();

    // pub fn estimate_fee();
}