use crate::error::Error;

use bdk::wallet::Update as BdkUpdate;
use bdk::Wallet as BdkWallet;
use bdk_esplora::esplora_client::AsyncClient as AsyncEsploraClient;
use miniscript::bitcoin::Transaction;

use std::{collections::HashMap, io::Write};

use bdk_esplora::{esplora_client, EsploraAsyncExt};

const STOP_GAP: usize = 50;
const PARALLEL_REQUESTS: usize = 5;
pub struct Client(AsyncEsploraClient);

impl Client {
    pub fn new(url: Option<String>) -> Result<Self, Error> {
        let url = match url {
            Some(url) => url.clone(),
            _ => "https://mempool.space/testnet/api".to_string(),
        };

        let client = esplora_client::Builder::new(&url)
            .build_async()
            .map_err(|_| Error::Generic {
                msg: "Could not create client".to_string(),
            })?;

        Ok(Client(client))
    }

    pub async fn scan(&self, wallet: &mut BdkWallet) -> Result<(), Error> {
        print!("Syncing...");
        // Scanning the blockchain

        let checkpoint = wallet.latest_checkpoint();
        let keychain_spks = wallet
            .spks_of_all_keychains()
            .into_iter()
            .map(|(k, k_spks)| {
                let mut once = Some(());
                let mut stdout = std::io::stdout();
                let k_spks = k_spks
                    .inspect(move |(spk_i, _)| match once.take() {
                        Some(_) => print!("\nScanning keychain [{:?}]", k),
                        None => print!(" {:<3}", spk_i),
                    })
                    .inspect(move |_| stdout.flush().expect("must flush"));
                (k, k_spks)
            })
            .collect();

        let (update_graph, last_active_indices) = self
            .0
            .scan_txs_with_keychains(keychain_spks, None, None, STOP_GAP, PARALLEL_REQUESTS)
            .await
            .map_err(|_| Error::Generic {
                msg: "Could not scan".to_string(),
            })?;

        let missing_heights = update_graph.missing_heights(wallet.local_chain());
        let chain_update = self
            .0
            .update_local_chain(checkpoint, missing_heights)
            .await
            .map_err(|_| Error::Generic {
                msg: "Could not update chain locally".to_string(),
            })?;

        let update = BdkUpdate {
            last_active_indices,
            graph: update_graph,
            chain: Some(chain_update),
        };

        wallet.apply_update(update).map_err(|_| Error::Generic {
            msg: "Couldn't apply wallet sync update".to_string(),
        })?;

        wallet.commit().map_err(|_| Error::Generic {
            msg: "Couldn't commit wallet sync update".to_string(),
        })?;

        Ok(())
    }

    pub async fn get_fees_estimation(&self) -> Result<HashMap<String, f64>, Error> {
        let fees = self
            .0
            .get_fee_estimates()
            .await
            .map_err(|_| Error::CannotGetFeeEstimation)?;

        Ok(fees)
    }

    pub async fn broadcast(&self, transaction: Transaction) -> Result<(), Error> {
        self.0
            .broadcast(&transaction)
            .await
            .map_err(|e| Error::Generic { msg: e.to_string() })
    }
}
