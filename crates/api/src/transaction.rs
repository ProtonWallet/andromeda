use std::{collections::HashMap, sync::Arc};

use async_std::sync::RwLock;
use bitcoin::{consensus::deserialize, Transaction};
use muon::{
    request::{Error as ReqError, Method, ProtonRequest, Response},
    session::Session,
};
use serde::{Deserialize, Serialize};

use super::{error::Error, BASE_WALLET_API_V1};

#[derive(Clone)]
pub struct TransactionClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
struct BroadcastRawTransactionRequestBody {
    SignedTransactionHex: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct BroadcastRawTransactionResponseBody {
    pub Code: u16,
    pub TransactionId: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetRawTransactionResponseBody {
    pub Code: u16,
    pub Details: (),
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct TransactionStatus {
    pub IsConfirmed: u8,
    pub BlockHeight: u32,
    pub BlockHash: String,
    pub BlockTime: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTransactionStatusResponseBody {
    pub Code: u16,
    pub TransactionStatus: TransactionStatus,
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
    pub Code: u16,
    pub Proof: TransactionMerkleProof,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTransactionMerkleBlockProofResponseBody {
    pub Code: u16,
    pub PartialMerkleTree: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct OutpointSpendingStatus {
    pub IsSpent: u8,
    pub TransactionId: String,
    pub Vin: u64,
    pub TransactionStatus: TransactionStatus,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetOutpointSpendingStatusResponseBody {
    pub Code: u16,
    pub Outspend: OutpointSpendingStatus,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetFeeEstimateResponseBody {
    pub Code: u16,
    pub FeeEstimates: HashMap<String, f64>,
}

impl TransactionClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn broadcast_raw_transaction(&self, signed_transaction_hex: String) -> Result<String, Error> {
        let body = BroadcastRawTransactionRequestBody {
            SignedTransactionHex: signed_transaction_hex,
        };

        let request = ProtonRequest::new(Method::POST, format!("{}/transactions", BASE_WALLET_API_V1))
            .json_body(body)
            .map_err(|e| e.into())?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;
        let parsed = response
            .to_json::<BroadcastRawTransactionResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.TransactionId)
    }

    pub async fn get_raw_transaction(&self, txid: String) -> Result<Transaction, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/transactions/{}/raw", BASE_WALLET_API_V1, txid));

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed: Transaction = deserialize(response.body()).map_err(|_| Error::DeserializeError)?;

        Ok(parsed)
    }

    pub async fn get_transaction_status(&self, txid: String) -> Result<TransactionStatus, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/{}/status", BASE_WALLET_API_V1, txid),
        );

        let response = self.session.read().await.bind(request).map_err(|e| e.into())?.send().await.map_err(|e| e.into())?;
        let parsed = response.to_json::<GetTransactionStatusResponseBody>().map_err(|e| e.into())?;

        Ok(parsed.TransactionStatus)
    }

    pub async fn get_transaction_merkle_proof(&self, txid: String) -> Result<TransactionMerkleProof, ReqError> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/{}/merkle-proof", BASE_WALLET_API_V1, txid),
        );

        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.to_json::<GetTransactionMerkleProofResponseBody>()?;

        Ok(parsed.Proof)
    }

    pub async fn get_transaction_merkle_block_proof(&self, txid: String) -> Result<String, ReqError> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/{}/merkleblock-proof", BASE_WALLET_API_V1, txid),
        );

        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.to_json::<GetTransactionMerkleBlockProofResponseBody>()?;

        Ok(parsed.PartialMerkleTree)
    }

    pub async fn get_outpoint_spending_status(
        &self,
        txid: String,
        index: u64,
    ) -> Result<OutpointSpendingStatus, ReqError> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/{}/outspend/{}", BASE_WALLET_API_V1, txid, index),
        );

        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.to_json::<GetOutpointSpendingStatusResponseBody>()?;

        Ok(parsed.Outspend)
    }

    pub async fn get_fee_estimates(&self) -> Result<HashMap<String, f64>, ReqError> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/fee-estimates", BASE_WALLET_API_V1),
        );

        let response = self.session.read().await.bind(request)?.send().await?;
        let parsed = response.to_json::<GetFeeEstimateResponseBody>()?;

        Ok(parsed.FeeEstimates)
    }
}

#[cfg(test)]
mod tests {
    use super::TransactionClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_raw_transaction() {
        let session = common_session().await;
        let client = TransactionClient::new(session);

        let blocks = client
            .get_raw_transaction("72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string())
            .await;
        println!("request done: {:?}", blocks);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_transaction_status() {
        let session = common_session().await;
        let client = TransactionClient::new(session);

        let transaction_status = client
            .get_transaction_status("72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string())
            .await;
        println!("request done: {:?}", transaction_status);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_transaction_merkle_proof() {
        let session = common_session().await;
        let client = TransactionClient::new(session);

        let transaction_merkle_proof =
            client
                .get_transaction_merkle_proof(
                    "72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string(),
                )
                .await;

        println!("request done: {:?}", transaction_merkle_proof);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_transaction_merkle_block_proof() {
        let session = common_session().await;
        let client = TransactionClient::new(session);

        let transaction_merkle_block_proof =
            client
                .get_transaction_merkle_block_proof(
                    "72a2f1d87b412c8db06b39a5027e98644150fd8ab41a54b0be762383e4283407".to_string(),
                )
                .await;

        println!("request done: {:?}", transaction_merkle_block_proof);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_outpoint_spending_status() {
        let session = common_session().await;
        let client = TransactionClient::new(session);

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
        let session = common_session().await;
        let client = TransactionClient::new(session);

        let fee_estimates = client.get_fee_estimates().await;

        println!("request done: {:?}", fee_estimates);
    }
}
