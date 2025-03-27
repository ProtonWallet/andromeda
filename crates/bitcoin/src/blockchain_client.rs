use crate::account_syncer::PARALLEL_REQUESTS;
use crate::error::Error;
use andromeda_api::transaction::RecommendedFees;
use andromeda_api::{
    transaction::{BroadcastMessage, ExchangeRateOrTransactionTime},
    ProtonWalletApiClient,
};
use andromeda_esplora::{AsyncClient, EsploraAsyncExt};
use bdk_chain::spk_client::SyncRequest;
use bdk_chain::CheckPoint;
use bdk_wallet::{bitcoin::Transaction, chain::spk_client::SyncResponse};
use bitcoin::ScriptBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct BlockchainClient(AsyncClient);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct MinimumFees {
    pub MinimumBroadcastFee: f32,
    pub MinimumIncrementalFee: f32,
}

impl BlockchainClient {
    pub fn new(proton_api_client: Arc<ProtonWalletApiClient>) -> Self {
        let client = AsyncClient::from_client(proton_api_client);
        BlockchainClient(client)
    }

    pub fn inner(&self) -> &AsyncClient {
        &self.0
    }

    pub async fn sync_spks(&self, spks_to_sync: Vec<ScriptBuf>, cp: CheckPoint) -> Result<SyncResponse, Error> {
        let request = SyncRequest::builder().chain_tip(cp).spks(spks_to_sync);
        Ok(self.0.sync(request, PARALLEL_REQUESTS).await?)
    }

    /// Returns mempool minimum fee, minimum relay tx fee and incremental relay
    /// fee in sat/vB instead of BTC/kB
    pub async fn get_minimum_fees(&self) -> Result<MinimumFees, Error> {
        let mempool_info = self.0.get_mempool_info().await?;
        let minimum_broadcast_fee = f32::max(
            mempool_info.MempoolMinFee * 100000.0,
            mempool_info.MinRelayTxFee * 100000.0,
        );
        let minimum_incremental_fee = f32::max(minimum_broadcast_fee, mempool_info.IncrementalRelayFee * 100000.0);
        Ok(MinimumFees {
            MinimumBroadcastFee: minimum_broadcast_fee,
            MinimumIncrementalFee: minimum_incremental_fee,
        })
    }

    /// Returns fee estimations in a Map
    pub async fn get_fees_estimation(&self) -> Result<HashMap<String, f64>, Error> {
        Ok(self.0.get_fee_estimates().await?)
    }

    /// Returns recommended fees
    pub async fn get_recommended_fees(&self) -> Result<RecommendedFees, Error> {
        Ok(self.0.get_recommended_fees().await?)
    }

    /// Broadcasts a provided transaction
    #[allow(clippy::too_many_arguments)]
    pub async fn broadcast(
        &self,
        transaction: Transaction,
        wallet_id: String,
        wallet_account_id: String,
        label: Option<String>,
        exchange_rate_or_transaction_time: ExchangeRateOrTransactionTime,
        address_id: Option<String>,
        body: Option<String>,
        message: Option<BroadcastMessage>,
        recipients: Option<HashMap<String, String>>,
        is_anonymous: Option<u8>,
        is_paper_wallet: Option<u8>,
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
                message,
                recipients,
                is_anonymous,
                is_paper_wallet,
            )
            .await?;

        Ok(())
    }
}
