use std::{collections::HashMap, sync::Arc};

use bitcoin::{consensus::deserialize, Transaction};
use log::info;
use muon::Response;
use serde::{Deserialize, Serialize};

use super::{error::Error, BASE_WALLET_API_V1};
use crate::{
    core::{ProtonResponseExt, ToProtonRequest},
    ProtonWalletApiClient,
};

//TODO:: code need to be used. remove all #[allow(dead_code)]

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
    Subject: Option<String>,
    Body: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct BroadcastRawTransactionResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub TransactionId: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct GetRawTransactionResponseBody {
    pub Code: u16,
    pub Details: (),
}

#[derive(Debug, Deserialize)]
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
    pub TransactionId: Option<String>,
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

pub enum ExchangeRateOrTransactionTime {
    ExchangeRate(String),
    TransactionTime(String),
}

#[derive(Clone)]
pub struct TransactionClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl TransactionClient {
    pub fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    pub async fn broadcast_raw_transaction(
        &self,
        signed_transaction_hex: String,
        wallet_id: String,
        wallet_account_id: String,
        label: Option<String>,
        exchange_rate_or_transaction_time: ExchangeRateOrTransactionTime,
        address_id: Option<String>,
        subject: Option<String>,
        body: Option<String>,
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
            Subject: subject,
            Body: body,
        };

        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "transactions")
            .to_post_request()
            .json_body(body)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<BroadcastRawTransactionResponseBody>()?;

        Ok(parsed.TransactionId)
    }

    pub async fn get_raw_transaction(&self, txid: String) -> Result<Transaction, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, format!("transactions/{}/raw", txid))
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed: Transaction = deserialize(response.body())?;

        Ok(parsed)
    }

    pub async fn get_transaction_status(&self, txid: String) -> Result<ApiTransactionStatus, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, format!("transactions/{}/status", txid))
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTransactionStatusResponseBody>()?;

        Ok(parsed.TransactionStatus)
    }

    pub async fn get_transaction_merkle_proof(&self, txid: String) -> Result<TransactionMerkleProof, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, format!("transactions/{}/merkle-proof", txid))
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTransactionMerkleProofResponseBody>()?;

        Ok(parsed.Proof)
    }

    pub async fn get_transaction_merkle_block_proof(&self, txid: String) -> Result<String, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, format!("transactions/{}/merkleblock-proof", txid))
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTransactionMerkleBlockProofResponseBody>()?;

        Ok(parsed.PartialMerkleTree)
    }

    pub async fn get_outpoint_spending_status(
        &self,
        txid: String,
        index: u64,
    ) -> Result<OutpointSpendingStatus, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, format!("transactions/{}/outspend/{}", txid, index))
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetOutpointSpendingStatusResponseBody>()?;

        Ok(parsed.Outspend)
    }

    pub async fn get_fee_estimates(&self) -> Result<HashMap<String, f64>, Error> {
        let request = self
            .api_client
            .build_full_url(BASE_WALLET_API_V1, "transactions/fee-estimates".to_string())
            .to_get_request();

        let response = self.api_client.send(request).await?;
        info!("get_fee_estimates {:?}", String::from_utf8(response.body().to_vec()));
        let parsed = response.parse_response::<GetFeeEstimateResponseBody>()?;

        Ok(parsed.FeeEstimates)
    }
}

#[cfg(test)]
mod tests {
    use super::TransactionClient;
    use crate::tests::utils::common_api_client;

    #[tokio::test]
    #[ignore]
    async fn should_get_raw_transaction() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let blocks = client
            .get_raw_transaction("72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string())
            .await;
        println!("request done: {:?}", blocks);
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
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_fee_estimates() {
        let api_client = common_api_client().await;
        let client = TransactionClient::new(api_client);

        let fee_estimates = client.get_fee_estimates().await;

        println!("request done: {:?}", fee_estimates);
    }
}
