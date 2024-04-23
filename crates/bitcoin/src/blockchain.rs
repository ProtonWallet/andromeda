use std::collections::HashMap;

use bdk::{blockchain::EsploraBlockchain, database::BatchDatabase, SyncOptions, Wallet as BdkWallet};
use bitcoin::Transaction;

use crate::error::Error;

const DEFAULT_URL: &'static str = "https://mempool.space/testnet/api";
const DEFAULT_STOP_GAP: usize = 10;

pub struct Blockchain(EsploraBlockchain);

impl Blockchain {
    pub fn new(url: Option<String>, stop_gap: Option<usize>) -> Self {
        let esplora = EsploraBlockchain::new(
            &url.unwrap_or(DEFAULT_URL.to_string()),
            stop_gap.unwrap_or(DEFAULT_STOP_GAP),
        );

        Blockchain(esplora)
    }

    /// Perform a full sync for the account
    pub async fn full_sync<D>(&self, wallet: &BdkWallet<D>) -> Result<(), Error>
    where
        D: BatchDatabase,
    {
        wallet.sync(&self.0, SyncOptions::default()).await?;

        Ok(())
    }

    /// Returns fee estimations in a Map
    pub async fn get_fees_estimation(&self) -> Result<HashMap<String, f64>, Error> {
        let fees = self.0.get_fee_estimates().await?;

        Ok(fees)
    }

    /// Broadcasts a provided transaction
    pub async fn broadcast(&self, transaction: Transaction) -> Result<(), Error> {
        self.0.broadcast(&transaction).await?;

        Ok(())
    }
}
