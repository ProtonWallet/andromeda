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

    use bitcoin::BlockHash;

    use super::BlockClient;
    use crate::{core::ApiClient, tests::utils::common_api_client};

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
}
