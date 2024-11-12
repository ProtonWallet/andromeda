use std::{collections::HashMap, sync::Arc};

use bitcoin::{consensus::deserialize, Transaction};
use serde::{Deserialize, Serialize};

use super::{error::Error, BASE_WALLET_API_V1};
use crate::{
    address::ApiTx,
    core::{ApiClient, ProtonResponseExt},
    ProtonWalletApiClient,
};

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct BroadcastMessage {
    pub Encrypted: String,
    pub Asymmetric: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct BroadcastRawTransactionRequestBody {
    SignedTransactionHex: String,
    WalletID: String,
    WalletAccountID: String,
    Label: Option<String>,
    ExchangeRateID: Option<String>,
    AddressID: Option<String>,
    TransactionTime: Option<String>,
    Body: Option<String>,
    Message: Option<BroadcastMessage>,
    Recipients: Option<HashMap<String, String>>,
    IsAnonymous: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct BroadcastRawTransactionResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub TransactionID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct GetRawTransactionResponseBody {
    pub Code: u16,
    pub Details: (),
}

#[derive(Clone, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiTransactionStatus {
    pub IsConfirmed: u8,
    pub BlockHeight: Option<u32>,
    pub BlockHash: Option<String>,
    pub BlockTime: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTransactionStatusResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub TransactionStatus: ApiTransactionStatus,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTransactionInfoResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Transaction: ApiTx,
}
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct TransactionMerkleProof {
    pub BlockHeight: u32,
    pub Merkle: Vec<String>,
    pub Position: u16,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTransactionMerkleProofResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Proof: TransactionMerkleProof,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTransactionMerkleBlockProofResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub PartialMerkleTree: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct OutpointSpendingStatus {
    pub IsSpent: u8,
    pub TransactionID: Option<String>,
    pub Vin: Option<u64>,
    pub TransactionStatus: Option<ApiTransactionStatus>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetOutpointSpendingStatusResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Outspend: OutpointSpendingStatus,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetFeeEstimateResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub FeeEstimates: HashMap<String, f64>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct MempoolInfo {
    pub Loaded: u8,
    pub Size: u32,
    pub Bytes: u32,
    pub Usage: u32,
    pub MaxMempool: u32,
    pub MempoolMinFee: f32,
    pub MinRelayTxFee: f32,
    pub IncrementalRelayFee: f32,
    pub UnbroadcastCount: u8,
    pub FullRbf: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetMempoolInfoResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub MempoolInfo: MempoolInfo,
}

pub enum ExchangeRateOrTransactionTime {
    ExchangeRate(String),
    TransactionTime(String),
}

#[derive(Clone)]
pub struct TransactionClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for TransactionClient {
    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }

    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }
}

impl TransactionClient {
    #[allow(clippy::too_many_arguments)]
    pub async fn broadcast_raw_transaction(
        &self,
        signed_transaction_hex: String,
        wallet_id: String,
        wallet_account_id: String,
        label: Option<String>,
        exchange_rate_or_transaction_time: ExchangeRateOrTransactionTime,
        address_id: Option<String>,
        body: Option<String>,
        message: Option<BroadcastMessage>,
        recipients: Option<HashMap<String, String>>,
        is_anonymous: Option<u8>,
    ) -> Result<String, Error> {
        let (exchange_rate_id, transaction_time) = match exchange_rate_or_transaction_time {
            ExchangeRateOrTransactionTime::ExchangeRate(exchange_rate) => (Some(exchange_rate), None),
            ExchangeRateOrTransactionTime::TransactionTime(transaction_time) => (None, Some(transaction_time)),
        };

        let body = BroadcastRawTransactionRequestBody {
            SignedTransactionHex: signed_transaction_hex,
            WalletID: wallet_id,
            WalletAccountID: wallet_account_id,
            Label: label,
            ExchangeRateID: exchange_rate_id,
            TransactionTime: transaction_time,
            AddressID: address_id,
            Body: body,
            Message: message,
            Recipients: recipients,
            IsAnonymous: is_anonymous.unwrap_or(0),
        };

        let request = self.post("transactions").body_json(body)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<BroadcastRawTransactionResponseBody>()?;

        Ok(parsed.TransactionID)
    }

    pub async fn get_raw_transaction(&self, txid: String) -> Result<Transaction, Error> {
        let request = self.get(format!("transactions/{}/raw", txid));

        let response = self.api_client.send(request).await?;
        let parsed: Transaction = deserialize(response.body())?;

        Ok(parsed)
    }

    pub async fn get_transaction_status(&self, txid: String) -> Result<ApiTransactionStatus, Error> {
        let request = self.get(format!("transactions/{}/status", txid));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTransactionStatusResponseBody>()?;

        Ok(parsed.TransactionStatus)
    }

    pub async fn get_transaction_info(&self, txid: String) -> Result<Option<ApiTx>, Error> {
        let request = self.get(format!("transactions/{}/info", txid));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTransactionInfoResponseBody>();

        match parsed {
            Ok(parsed) => Ok(Some(parsed.Transaction)),
            Err(Error::ErrorCode(_, error)) if error.Code == 2001 => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn get_transaction_merkle_proof(&self, txid: String) -> Result<TransactionMerkleProof, Error> {
        let request = self.get(format!("transactions/{}/merkle-proof", txid));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTransactionMerkleProofResponseBody>()?;

        Ok(parsed.Proof)
    }

    pub async fn get_transaction_merkle_block_proof(&self, txid: String) -> Result<String, Error> {
        let request = self.get(format!("transactions/{}/merkleblock-proof", txid));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTransactionMerkleBlockProofResponseBody>()?;

        Ok(parsed.PartialMerkleTree)
    }

    pub async fn get_outpoint_spending_status(
        &self,
        txid: String,
        index: u64,
    ) -> Result<OutpointSpendingStatus, Error> {
        let request = self.get(format!("transactions/{}/outspend/{}", txid, index));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetOutpointSpendingStatusResponseBody>()?;

        Ok(parsed.Outspend)
    }

    pub async fn get_fee_estimates(&self) -> Result<HashMap<String, f64>, Error> {
        let request = self.get("transactions/fee-estimates");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetFeeEstimateResponseBody>()?;

        Ok(parsed.FeeEstimates)
    }

    pub async fn get_mempool_info(&self) -> Result<MempoolInfo, Error> {
        let request = self.get("mempool/info");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetMempoolInfoResponseBody>()?;

        Ok(parsed.MempoolInfo)
    }
}

#[cfg(test)]
mod tests {
    use super::TransactionClient;
    use crate::{core::ApiClient, tests::utils::common_api_client};

    #[tokio::test]
    #[ignore]
    async fn should_get_raw_transaction() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let blocks = client
            .get_raw_transaction("72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string())
            .await;
        println!("request done: {:?}", blocks);
        assert!(blocks.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_transaction_status() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let transaction_status = client
            .get_transaction_status("72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string())
            .await;
        println!("request done: {:?}", transaction_status);
        assert!(transaction_status.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_transaction_merkle_proof() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let transaction_merkle_proof = client
            .get_transaction_merkle_proof(
                "72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string(),
            )
            .await;

        println!("request done: {:?}", transaction_merkle_proof);
        assert!(transaction_merkle_proof.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_transaction_merkle_block_proof() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let transaction_merkle_block_proof = client
            .get_transaction_merkle_block_proof(
                "72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string(),
            )
            .await;

        println!("request done: {:?}", transaction_merkle_block_proof);
        assert!(transaction_merkle_block_proof.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_outpoint_spending_status() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let outpoint_spending_status = client
            .get_outpoint_spending_status(
                "72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string(),
                0,
            )
            .await;

        println!("request done: {:?}", outpoint_spending_status);
        assert!(outpoint_spending_status.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_fee_estimates() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let fee_estimates = client.get_fee_estimates().await;

        println!("request done: {:?}", fee_estimates);
        assert!(fee_estimates.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_mempool_info() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let mempool_info = client.get_mempool_info().await;

        println!("request done: {:?}", mempool_info);
        assert!(mempool_info.is_ok());
    }
}
