use std::{collections::HashMap, sync::Arc};

use async_std::sync::RwLock;
use bitcoin::{consensus::deserialize, Transaction};
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::{Deserialize, Serialize};

use super::{error::Error, BASE_WALLET_API_V1};

//TODO:: code need to be used. remove all #[allow(dead_code)]

#[derive(Clone)]
pub struct TransactionClient {
    session: Arc<RwLock<Session>>,
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
pub struct TransactionStatus {
    pub IsConfirmed: u8,
    pub BlockHeight: u32,
    pub BlockHash: String,
    pub BlockTime: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTransactionStatusResponseBody {
    #[allow(dead_code)]
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
    pub TransactionId: String,
    pub Vin: u64,
    pub TransactionStatus: TransactionStatus,
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

impl TransactionClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn broadcast_raw_transaction(
        &self,
        signed_transaction_hex: String,
        wallet_id: String,
        wallet_account_id: String,
        label: Option<String>,
        exchange_rate_id: Option<String>,
        transaction_time: Option<String>,
        address_id: Option<String>,
        subject: Option<String>,
        body: Option<String>,
    ) -> Result<String, Error> {
        // need to pass at least transaction_time or exchange_rate_id
        // backend will check ts from them
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

        let utf8_str = std::str::from_utf8(response.body()).unwrap();
        println!("broadcast_raw_transaction() response: {}", utf8_str);

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
            .to_json::<GetTransactionStatusResponseBody>()
            .map_err(|e| e.into())?;

        Ok(parsed.TransactionStatus)
    }

    pub async fn get_transaction_merkle_proof(&self, txid: String) -> Result<TransactionMerkleProof, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/{}/merkle-proof", BASE_WALLET_API_V1, txid),
        );

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
            .to_json::<GetTransactionMerkleProofResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Proof)
    }

    pub async fn get_transaction_merkle_block_proof(&self, txid: String) -> Result<String, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/{}/merkleblock-proof", BASE_WALLET_API_V1, txid),
        );

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
            .to_json::<GetTransactionMerkleBlockProofResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.PartialMerkleTree)
    }

    pub async fn get_outpoint_spending_status(
        &self,
        txid: String,
        index: u64,
    ) -> Result<OutpointSpendingStatus, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/{}/outspend/{}", BASE_WALLET_API_V1, txid, index),
        );

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
            .to_json::<GetOutpointSpendingStatusResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Outspend)
    }

    pub async fn get_fee_estimates(&self) -> Result<HashMap<String, f64>, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/transactions/fee-estimates", BASE_WALLET_API_V1),
        );

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
            .to_json::<GetFeeEstimateResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

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
        let session = common_session().await;
        let client = TransactionClient::new(session);

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
