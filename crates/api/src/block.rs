use std::{str::FromStr, sync::Arc};

use bitcoin::{block::Header as BlockHeader, consensus::deserialize, hashes::hex::FromHex, Block, BlockHash};
use serde::Deserialize;

use super::BASE_WALLET_API_V1;
use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient,
};

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiBlock {
    pub ID: String,
    pub BlockHeight: u32,
    pub Version: u64,
    pub Timestamp: u64,
    pub TxCount: u32,
    pub Size: u64,
    pub Weight: u64,
    pub MerkleRoot: String,
    pub PreviousBlockHash: Option<String>,
    pub MedianTime: u64,
    pub Nonce: u64,
    pub Bits: u64,
    pub Difficulty: f32,
}

//TODO:: code need to be used. remove all #[allow(dead_code)]

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBlocksResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Blocks: Vec<ApiBlock>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetHeaderByHashResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub BlockHeader: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBlockHashByBlockHeightResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub BlockHash: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct BlockStatus {
    pub IsInBestChain: u8,
    pub BlockHeight: u32,
    pub NextBest: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBlockStatusResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub BlockStatus: BlockStatus,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case, dead_code)]
struct GetBlockByHashResponseBody {
    pub Code: u16,
    pub Details: Block,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTxIdAtBlockIndexResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub TransactionID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTipHeightResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Height: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTipHashResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub BlockHash: String,
}

#[derive(Clone)]
pub struct BlockClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for BlockClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }
}

impl BlockClient {
    /// Get recent block summaries, starting at tip or height if provided
    pub async fn get_blocks(&self, height: Option<u32>) -> Result<Vec<ApiBlock>, Error> {
        let request = self.get(match height {
            Some(height) => format!("blocks/{}", height),
            None => "blocks".to_string(),
        });
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBlocksResponseBody>()?;

        Ok(parsed.Blocks)
    }

    /// Get a [`BlockHeader`] given a particular block hash.
    pub async fn get_header_by_hash(&self, block_hash: &BlockHash) -> Result<BlockHeader, Error> {
        let request = self.get(format!("blocks/{}/header", block_hash));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetHeaderByHashResponseBody>()?;

        Ok(deserialize(&Vec::from_hex(&parsed.BlockHeader)?)?)
    }

    pub async fn get_block_hash(&self, block_height: u32) -> Result<BlockHash, Error> {
        let request = self.get(format!("blocks/height/{}/hash", block_height));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBlockHashByBlockHeightResponseBody>()?;

        Ok(BlockHash::from_str(&parsed.BlockHash)?)
    }

    pub async fn get_block_status(&self, block_hash: &BlockHash) -> Result<BlockStatus, Error> {
        let request = self.get(format!("blocks/{}/status", block_hash));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBlockStatusResponseBody>()?;

        Ok(parsed.BlockStatus)
    }

    pub async fn get_block_by_hash(&self, block_hash: &BlockHash) -> Result<Block, Error> {
        let request = self.get(format!("blocks/{}/raw", block_hash));

        let response = self.api_client.send(request).await?;

        Ok(deserialize(response.body())?)
    }

    pub async fn get_txid_at_block_index(&self, block_hash: &BlockHash, index: usize) -> Result<String, Error> {
        let request = self.get(format!("blocks/{}/txid/{}", block_hash, index));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTxIdAtBlockIndexResponseBody>()?;
        Ok(parsed.TransactionID)
    }

    pub async fn get_tip_height(&self) -> Result<u32, Error> {
        let request = self.get("blocks/tip/height");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTipHeightResponseBody>()?;
        Ok(parsed.Height)
    }

    pub async fn get_tip_hash(&self) -> Result<BlockHash, Error> {
        let request = self.get("blocks/tip/hash");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetTipHashResponseBody>()?;

        Ok(BlockHash::from_str(&parsed.BlockHash)?)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{
        block::{Header as BlockHeader, Version},
        hex::{Case, DisplayHex},
        BlockHash, CompactTarget, TxMerkleNode,
    };

    use super::BlockClient;

    use crate::{
        core::ApiClient, read_mock_file, read_mock_raw_file, tests::utils::common_api_client,
        tests::utils::setup_test_connection, BASE_WALLET_API_V1,
    };
    use std::sync::Arc;
    use wiremock::{
        matchers::{method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_blocks() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let blocks = client.get_blocks(Some(0u32)).await;
        println!("request done: {:?}", blocks);
        assert!(blocks.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_header_by_hash() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let header = client
            .get_header_by_hash(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
            )
            .await;
        println!("request done: {:?}", header);
        assert!(header.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_block_hash() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let block_hash = client.get_block_hash(0u32).await;
        println!("request done: {:?}", block_hash);
        assert!(block_hash.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_block_status() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let block_status = client
            .get_block_status(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
            )
            .await;
        println!("request done: {:?}", block_status);
        assert!(block_status.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_block_by_hash() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let block = client
            .get_block_by_hash(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
            )
            .await;
        println!("request done: {:?}", block);
        assert!(block.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_txid_at_block_index() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let txid = client
            .get_txid_at_block_index(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
                0,
            )
            .await;
        println!("request done: {:?}", txid);
        assert!(txid.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_tip_height() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let header = client.get_tip_height().await;
        println!("request done: {:?}", header);
        assert!(header.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_tip_hash() {
        let api_client = common_api_client().await;
        let client = BlockClient::new(api_client);

        let block_hash = client.get_tip_hash().await;
        println!("request done: {:?}", block_hash);
        assert!(block_hash.is_ok());
    }

    #[tokio::test]
    async fn test_get_blocks_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("get_blocks_1000_body");
        assert!(!contents.is_empty());
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let req_path: String = format!("{}/blocks", BASE_WALLET_API_V1);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_blocks(None).await;
        match result {
            Ok(blocks) => {
                assert_eq!(
                    blocks[0].ID,
                    "000000000000000000013b1489869b5537ef7d3880be22f713258f1cb83f0f10"
                );
                assert_eq!(blocks[0].BlockHeight, 871864);
                assert_eq!(blocks[0].Version, 541065216);
                assert_eq!(blocks[0].Timestamp, 1732516627);
                assert_eq!(blocks[0].TxCount, 6010);
                assert_eq!(blocks[0].Size, 1459737);
                assert_eq!(blocks[0].Weight, 3993453);
                assert_eq!(
                    blocks[0].MerkleRoot,
                    "775c14a1a51ba71556177b81073bf0c6c707cc56da10f368f3a023f68f64a878"
                );
                assert_eq!(
                    blocks[0].PreviousBlockHash,
                    Some("0000000000000000000084855fd06ffed9ac4cf7df513e09ed1011522b67275c".to_string())
                );
                assert_eq!(blocks[0].MedianTime, 1732512598);
                assert_eq!(blocks[0].Nonce, 3131843720);
                assert_eq!(blocks[0].Bits, 386056304);
                assert_eq!(blocks[0].Difficulty, 102289407543323.8);

                assert_eq!(
                    blocks[1].ID,
                    "0000000000000000000084855fd06ffed9ac4cf7df513e09ed1011522b67275c"
                );
                assert_eq!(blocks[1].BlockHeight, 871863);
                assert_eq!(blocks[1].Version, 602308608);
                assert_eq!(blocks[1].Timestamp, 1732516010);
                assert_eq!(blocks[1].TxCount, 4666);
                assert_eq!(blocks[1].Size, 1559248);
                assert_eq!(blocks[1].Weight, 3993451);
                assert_eq!(
                    blocks[1].MerkleRoot,
                    "809b8c389eb82ee8479187b6c90e325948ae722d7c069469e62278e81395e398"
                );
                assert_eq!(
                    blocks[1].PreviousBlockHash,
                    Some("00000000000000000002811d58f6671c30c7a758cd89aa855060b42c1d1aec31".to_string())
                );
                assert_eq!(blocks[1].MedianTime, 1732511761);
                assert_eq!(blocks[1].Nonce, 1794818178);
                assert_eq!(blocks[1].Bits, 386056304);
                assert_eq!(blocks[1].Difficulty, 102289407543323.8);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_header_by_hash_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("get_blocks_header_by_hash_1000_body");
        assert!(!contents.is_empty());
        let block_hash =
            BlockHash::from_str("0000000000000000000084855fd06ffed9ac4cf7df513e09ed1011522b67275c").unwrap();
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let req_path: String = format!("{}/blocks/{}/header", BASE_WALLET_API_V1, block_hash);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_header_by_hash(&block_hash).await;
        match result {
            Ok(block) => {
                assert_eq!(block.version, Version::from_consensus(541065216));
                assert_eq!(
                    block.prev_blockhash.to_string(),
                    "0000000000000000000084855fd06ffed9ac4cf7df513e09ed1011522b67275c"
                );
                assert_eq!(
                    block.merkle_root.to_string(),
                    "775c14a1a51ba71556177b81073bf0c6c707cc56da10f368f3a023f68f64a878"
                );
                assert_eq!(block.time, 1732516627);
                assert_eq!(block.bits, CompactTarget::from_consensus(386056304));
                assert_eq!(block.nonce, 3131843720);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_block_hash_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("get_block_hash_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let block_height = 20241125;
        let req_path: String = format!("{}/blocks/height/{}/hash", BASE_WALLET_API_V1, block_height);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_block_hash(block_height).await;
        match result {
            Ok(block_hash) => {
                assert_eq!(
                    block_hash.to_string(),
                    "00000000000000000000b3f5fed64a5eefd02589a036ce9f9bd40b627b4cb9a3"
                );
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_block_status_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("get_block_status_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let block_hash =
            BlockHash::from_str("00000000000000000000b3f5fed64a5eefd02589a036ce9f9bd40b627b4cb9a3").unwrap();
        let req_path: String = format!("{}/blocks/{}/status", BASE_WALLET_API_V1, block_hash);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_block_status(&block_hash).await;
        match result {
            Ok(block_status) => {
                assert_eq!(block_status.IsInBestChain, 1);
                assert_eq!(block_status.BlockHeight, 871864);
                assert_eq!(
                    block_status.NextBest,
                    "0000000000000000000084855fd06ffed9ac4cf7df513e09ed1011522b67275c".to_string()
                );
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_block_by_hash_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_raw_file!("get_block_by_hash");

        let response = ResponseTemplate::new(200).set_body_bytes(contents);
        let block_hash =
            BlockHash::from_str("00000000000000000000b3f5fed64a5eefd02589a036ce9f9bd40b627b4cb9a3").unwrap();
        let req_path: String = format!("{}/blocks/{}/raw", BASE_WALLET_API_V1, block_hash);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_block_by_hash(&block_hash).await;
        match result {
            Ok(block) => {
                assert_eq!(block.header.version, Version::from_consensus(541065216));
                assert_eq!(
                    block.header.prev_blockhash.to_string(),
                    "0000000000000000000084855fd06ffed9ac4cf7df513e09ed1011522b67275c"
                );
                assert_eq!(
                    block.header.merkle_root.to_string(),
                    "775c14a1a51ba71556177b81073bf0c6c707cc56da10f368f3a023f68f64a878"
                );
                assert_eq!(block.header.time, 1732516627);
                assert_eq!(block.header.bits, CompactTarget::from_consensus(386056304));
                assert_eq!(block.header.nonce, 3131843720);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_txid_at_block_index_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({
            "Code": 1000,
            "TransactionID": "d463c5e0f89e55fa2af3e484e4bc94959ecf4aec9eb4136527f5854bc940ae51"
            }
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        let block_hash =
            BlockHash::from_str("00000000000000000000b3f5fed64a5eefd02589a036ce9f9bd40b627b4cb9a3").unwrap();
        let index: usize = 10;
        let req_path: String = format!("{}/blocks/{}/txid/{}", BASE_WALLET_API_V1, block_hash, index);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_txid_at_block_index(&block_hash, index).await;
        match result {
            Ok(txid) => {
                assert_eq!(
                    txid,
                    "d463c5e0f89e55fa2af3e484e4bc94959ecf4aec9eb4136527f5854bc940ae51".to_string()
                );
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_tip_height_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({
            "Code": 1000,
            "Height": 2024112515
            }
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        let req_path: String = format!("{}/blocks/tip/height", BASE_WALLET_API_V1,);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_tip_height().await;
        match result {
            Ok(height) => {
                assert_eq!(height, 2024112515);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_tip_hash_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("get_block_hash_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let req_path: String = format!("{}/blocks/tip/hash", BASE_WALLET_API_V1,);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockClient::new(Arc::new(api_client));
        let result = client.get_tip_hash().await;
        match result {
            Ok(block_hash) => {
                assert_eq!(
                    block_hash.to_string(),
                    "00000000000000000000b3f5fed64a5eefd02589a036ce9f9bd40b627b4cb9a3"
                );
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
