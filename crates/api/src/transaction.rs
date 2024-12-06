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
    pub DataPacket: String,
    pub KeyPackets: HashMap<String, String>,
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
pub struct RecommendedFees {
    /// Fee rate in sat/vB to place the transaction in the first mempool block
    pub FastestFee: u8,
    /// Fee rate in sat/vB to usually confirm within half hour and place the transaction in between the first and second mempool blocks
    pub HalfHourFee: u8,
    /// Fee rate in sat/vB to usually confirm within one hour and place the transaction in between the second and third mempool blocks
    pub HourFee: u8,
    /// Either 2 times the minimum fees, or the low priority rate (whichever is lower)
    pub EconomyFee: u8,
    /// Minimum fee rate in sat/vB for transaction to be accepted
    pub MinimumFee: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetRecommendedFeesResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub RecommendedFees: RecommendedFees,
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

    pub async fn get_recommended_fees(&self) -> Result<RecommendedFees, Error> {
        let request = self.get("fees/recommended");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetRecommendedFeesResponseBody>()?;

        Ok(parsed.RecommendedFees)
    }
}

#[cfg(test)]
mod tests {
    use super::TransactionClient;
    use crate::{
        core::ApiClient,
        read_mock_file, read_mock_raw_file,
        tests::utils::{common_api_client, setup_test_connection},
        transaction::ExchangeRateOrTransactionTime,
        BASE_WALLET_API_V1,
    };
    use bitcoin::transaction::Version;
    use std::{collections::HashMap, sync::Arc};
    use wiremock::{
        matchers::{body_json, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

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

    #[tokio::test]
    async fn test_get_raw_transaction_success() {
        let mock_server = MockServer::start().await;
        let txid = "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa";
        let req_path: String = format!("{}/transactions/{}/raw", BASE_WALLET_API_V1, txid);
        let contents = read_mock_raw_file!("get_transaction");
        let response = ResponseTemplate::new(200).set_body_bytes(contents);

        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_raw_transaction(txid.to_string()).await;
        match result {
            Ok(transaction) => {
                assert_eq!(transaction.version, Version::ONE);
                assert_eq!(
                    transaction.input[0].previous_output.txid.to_string(),
                    "5a27a55abbb08d1f10474125735fcfe81043d339202d0a67767c09789cd96d7a"
                );
                assert_eq!(transaction.output[0].value.to_sat(), 40520);
                assert_eq!(transaction.output[1].value.to_sat(), 373759);
                assert_eq!(transaction.vsize(), 144);
                assert_eq!(
                    transaction.compute_txid().to_string(),
                    "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa"
                );
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_transaction_status_success() {
        let mock_server = MockServer::start().await;
        let txid = "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa";
        let req_path: String = format!("{}/transactions/{}/status", BASE_WALLET_API_V1, txid);
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "TransactionStatus": {
                    "IsConfirmed": 0,
                    "BlockHeight": 2024112517,
                    "BlockHash": "blockhash",
                    "BlockTime": 214600000,
                }
            }
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_transaction_status(txid.to_string()).await;
        match result {
            Ok(status) => {
                assert_eq!(status.IsConfirmed, 0);
                assert_eq!(status.BlockHeight.unwrap(), 2024112517);
                assert_eq!(status.BlockHash.unwrap(), "blockhash");
                assert_eq!(status.BlockTime.unwrap(), 214600000);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_transaction_info_success() {
        let mock_server = MockServer::start().await;
        let txid = "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa";
        let req_path: String = format!("{}/transactions/{}/info", BASE_WALLET_API_V1, txid);

        let contents = read_mock_file!("get_transaction_info_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_transaction_info(txid.to_string()).await;
        match result {
            Ok(api_tx) => {
                assert!(api_tx.is_some());
                let api_tx = api_tx.unwrap();
                assert_eq!(
                    api_tx.TransactionID,
                    "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa"
                );
                assert_eq!(api_tx.Version, 1);
                assert_eq!(api_tx.Locktime, 0);
                assert_eq!(api_tx.Size, 225);
                assert_eq!(api_tx.Weight, 573);
                assert_eq!(api_tx.Fee, 1420);
                assert_eq!(api_tx.TransactionStatus.IsConfirmed, 1);
                assert_eq!(api_tx.TransactionStatus.BlockHeight.unwrap(), 871877);
                assert_eq!(
                    api_tx.TransactionStatus.BlockHash.unwrap(),
                    "00000000000000000001fc76f15612a788e9aa1b4921edf9cd871ab4fec251dd"
                );
                assert_eq!(api_tx.TransactionStatus.BlockTime.unwrap(), 1732527325);
                assert_eq!(api_tx.Vin.unwrap().len(), 1);
                assert_eq!(api_tx.Vout.unwrap().len(), 2);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_transaction_merkle_success() {
        let mock_server = MockServer::start().await;
        let txid = "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa";
        let req_path: String = format!("{}/transactions/{}/merkle-proof", BASE_WALLET_API_V1, txid);
        let contents = read_mock_file!("get_transaction_merkle_proof_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);

        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_transaction_merkle_proof(txid.to_string()).await;
        match result {
            Ok(merkle) => {
                assert_eq!(merkle.BlockHeight, 871877);
                assert_eq!(merkle.Merkle.len(), 12);
                assert_eq!(merkle.Position, 1136);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_transaction_merkle_block_proof_success() {
        let mock_server = MockServer::start().await;
        let txid = "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa";
        let req_path: String = format!("{}/transactions/{}/merkleblock-proof", BASE_WALLET_API_V1, txid);
        let partial_merkle_tree = "00a00020312c5cc29c13a67b2378f29e44f8c226799f6033693c01000000000000000000636a014ae1e5e9be17cf7424ec1d543de07e4634fb1ad4f211dd885cdf86ec7ddd44446770c0021760fb4984750d00000d36f2cf75c4066b9de7a431231555731cdefe6e585b3612e99aa58c1f0b6da8260f1b33fda7fa5749849168e75389da579632ab02d4a7ae5297790d0ea7a9885fd7e950ed07991a456862f3373efa6c4875877d2c13a7c49ff19a465d1ebdba850d33f75d5f918b868de0e005108e3657ce90ab86d300c67ed09458e16277df40fa90f64f5445aeb59310f28ec752eeb329378ffa5011feffb2e411f96ec0bf6b03b3dcdd4c7aeecadd1598dfeab1a06b210d7f8aca13f5e0cf1ae98efcf43ab3580ae5b511f6f77f08ca781b7a8eae57fefe9729f730d2a1f8ca1364a80f73f6e5a0edccbc65778f24d009a8a29792db625606caf0f995b62dc03a19f5c72cefb3cdb84295db345b4a657bc31825235edb2f928398401825c1aca8d4d470bd2ac071c207462f0c26967c6411c5ef19459cc581e0e844048ee908c396fcb2907bf304d925c3d7e6c882b483b4055d3e575d3f25790b7452bbbb477e1aaa701a78ac84f3d30466c4a343f0a8d8fd17b2c40d042c79341688a9dff3d9c8b17d5bc898c5729e2f2674859bfcc51f94ad3d50efd80294a88265f13623ff8416e3042d047bf50100";
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "PartialMerkleTree": partial_merkle_tree
            }
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_transaction_merkle_block_proof(txid.to_string()).await;
        match result {
            Ok(value) => {
                assert_eq!(value, partial_merkle_tree);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_outpoint_spending_status_success() {
        let mock_server = MockServer::start().await;
        let txid = "6bbfc06ef911e4b2fffe1150fa8f3729b3ee52c78ef21093b5ae45544ff690fa";
        let index = 0;
        let req_path: String = format!("{}/transactions/{}/outspend/{}", BASE_WALLET_API_V1, txid, index);
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "Outspend": {
                    "IsSpent": 0
                }
            }
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_outpoint_spending_status(txid.to_string(), index).await;
        match result {
            Ok(status) => {
                assert_eq!(status.IsSpent, 0);
                assert!(status.TransactionID.is_none());
                assert!(status.TransactionStatus.is_none());
                assert!(status.Vin.is_none());
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_fee_estimates_success() {
        let mock_server = MockServer::start().await;
        let req_path: String = format!("{}/transactions/fee-estimates", BASE_WALLET_API_V1);
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "FeeEstimates": {"15":3.4709999999999996,"12":5.4030000000000005,"19":3.149,"6":6.211,"3":6.551,"1":6.969,"5":6.211,"2":6.969,"8":5.4030000000000005,"9":5.4030000000000005,"18":3.149,"504":1.6580000000000001,"21":3.131,"7":5.4030000000000005,"11":5.4030000000000005,"4":6.211,"23":3.131,"25":3.131,"1008":1.2,"14":3.4709999999999996,"16":3.149,"22":3.131,"20":3.131,"144":2.239,"10":5.4030000000000005,"13":3.4709999999999996,"17":3.149,"24":3.131}
            }
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_fee_estimates().await;
        match result {
            Ok(fees) => {
                assert_eq!(fees.get("1").unwrap().abs(), 6.969);
                assert_eq!(fees.get("2").unwrap().abs(), 6.969);
                assert_eq!(fees.get("3").unwrap().abs(), 6.551);
                assert_eq!(fees.get("5").unwrap().abs(), 6.211);
                assert_eq!(fees.get("6").unwrap().abs(), 6.211);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_mempool_info_success() {
        let mock_server = MockServer::start().await;
        let req_path: String = format!("{}/mempool/info", BASE_WALLET_API_V1);
        let contents = read_mock_file!("get_mempool_info_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client.get_mempool_info().await;
        match result {
            Ok(info) => {
                assert_eq!(info.Loaded, 1);
                assert_eq!(info.Size, 58805);
                assert_eq!(info.Bytes, 33596529);
                assert_eq!(info.Usage, 194323824);
                assert_eq!(info.MaxMempool, 300000000);
                assert_eq!(info.MempoolMinFee, 0.00001);
                assert_eq!(info.MinRelayTxFee, 0.00001);
                assert_eq!(info.IncrementalRelayFee, 0.00001);
                assert_eq!(info.UnbroadcastCount, 0);
                assert_eq!(info.FullRbf, 0);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_broadcast_transaction_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "TransactionID": "f6e1136902960f7cc5b8f2d7a8206cc311841d278a9d5ddb4d536e5eaa53c725"
            }
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        let req_path: String = format!("{}/transactions", BASE_WALLET_API_V1);
        let signed_transaction_hex = "signed_transaction_hex";
        let wallet_id = "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==";
        let wallet_account_id =
            "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==";
        let exchange_rate_id =
            "L-sLJPOJL4LFpttI3tsgK9qyH24Mfw9VBJk6vlizZZrLNbpf7a_5Y_idfPH7UaOFWz8TJPpOaDcxd-pQNmKpOg==";
        let address_id = "v4f03EPBAzQEogZgQX68hpaNqNuXwKN6X1us0nrJCTA6Zt3SdozXUmEmxqceBO22CccjmOWtwFyFraTAH2LE8A==";
        let body = "-----BEGIN PGP MESSAGE-----*-----END PGP MESSAGE-----\n";
        let is_anonymous = 0;
        let mut recipient_map: HashMap<String, String> = HashMap::new();
        recipient_map.insert(
            "bc1qqkaa0f63navwd5lv0axsy3fdalxc4f03xqhn0c".to_string(),
            "test.1@proton.me".to_string(),
        );
        Mock::given(method("POST"))
            .and(path(req_path))
            // following constraint not sure why not work
            // .and(body_json(serde_json::json!(
            // {
            //     "SignedTransactionHex": signed_transaction_hex,
            //     "WalletID": wallet_id,
            //     "WalletAccountID": wallet_account_id,
            //     "Label": null,
            //     "ExchangeRateID": exchange_rate_id,
            //     "AddressID": address_id,
            //     "TransactionTime": null,
            //     "Body": body,
            //     // "Recipients": recipient_map,
            //     "IsAnonymous": is_anonymous
            // })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = TransactionClient::new(Arc::new(api_client));
        let result = client
            .broadcast_raw_transaction(
                signed_transaction_hex.to_string(),
                wallet_id.to_string(),
                wallet_account_id.to_string(),
                None,
                ExchangeRateOrTransactionTime::ExchangeRate(exchange_rate_id.to_string()),
                Some(address_id.to_string()),
                Some(body.to_string()),
                None,
                Some(recipient_map),
                Some(is_anonymous),
            )
            .await;
        match result {
            Ok(transaction_id) => {
                assert_eq!(
                    transaction_id,
                    "f6e1136902960f7cc5b8f2d7a8206cc311841d278a9d5ddb4d536e5eaa53c725"
                );
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
