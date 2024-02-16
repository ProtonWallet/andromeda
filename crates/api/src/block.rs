use std::{str::FromStr, sync::Arc};

use async_std::sync::RwLock;
use bitcoin::{block::Header as BlockHeader, consensus::deserialize, hashes::hex::FromHex, Block, BlockHash};
use muon::{
    request::{Method, ProtonRequest, Response},
    session::Session,
};
use serde::Deserialize;

use super::BASE_WALLET_API_V1;
use crate::error::Error;

pub struct BlockClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBlocksResponseBody {
    pub Code: u16,
    pub Blocks: Vec<Block>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetHeaderByHashResponseBody {
    pub Code: u16,
    pub BlockHeader: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBlockHashByBlockHeightResponseBody {
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
    pub Code: u16,
    pub BlockStatus: BlockStatus,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBlockByHashResponseBody {
    Code: u16,
    Details: Block,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTxIdAtBlockIndexResponseBody {
    pub Code: u16,
    pub TransactionId: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTipHeightResponseBody {
    pub Code: u16,
    pub Height: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetTipHashResponseBody {
    pub Code: u16,
    pub BlockHash: String,
}

impl BlockClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    /// Get recent block summaries, starting at tip or height if provided
    pub async fn get_blocks(&self, height: Option<u32>) -> Result<Vec<Block>, Error> {
        let url = match height {
            Some(height) => format!("{}/blocks/{}", BASE_WALLET_API_V1, height),
            None => format!("{}/blocks", BASE_WALLET_API_V1),
        };

        let request = ProtonRequest::new(Method::GET, url);
        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        println!("{:?}", response.to_json::<serde_json::Value>().unwrap());

        let parsed = response
            .to_json::<GetBlocksResponseBody>()
            .map_err(|_| Error::DeserializeError)?;
        Ok(parsed.Blocks)
    }

    /// Get a [`BlockHeader`] given a particular block hash.
    pub async fn get_header_by_hash(&self, block_hash: &BlockHash) -> Result<BlockHeader, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/blocks/{}/header", BASE_WALLET_API_V1, block_hash),
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
            .to_json::<GetHeaderByHashResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(
            deserialize(&Vec::from_hex(&parsed.BlockHeader).map_err(|_| Error::DeserializeError)?)
                .map_err(|_| Error::DeserializeError)?,
        )
    }

    pub async fn get_block_hash(&self, block_height: u32) -> Result<BlockHash, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/blocks/height/{}/hash", BASE_WALLET_API_V1, block_height),
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
            .to_json::<GetBlockHashByBlockHeightResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(BlockHash::from_str(&parsed.BlockHash).map_err(|_| Error::DeserializeError)?)
    }

    pub async fn get_block_status(&self, block_hash: &BlockHash) -> Result<BlockStatus, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/blocks/{}/status", BASE_WALLET_API_V1, block_hash),
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
            .to_json::<GetBlockStatusResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.BlockStatus)
    }

    pub async fn get_block_by_hash(&self, block_hash: &BlockHash) -> Result<Block, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/blocks/{}/raw", BASE_WALLET_API_V1, block_hash));

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        Ok(deserialize(response.body()).map_err(|_| Error::DeserializeError)?)
    }

    pub async fn get_txid_at_block_index(&self, block_hash: &BlockHash, index: usize) -> Result<String, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/blocks/{}/txid/{}", BASE_WALLET_API_V1, block_hash, index),
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
            .to_json::<GetTxIdAtBlockIndexResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.TransactionId)
    }

    pub async fn get_tip_height(&self) -> Result<u32, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/blocks/tip/height", BASE_WALLET_API_V1,));

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
            .to_json::<GetTipHeightResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Height)
    }

    pub async fn get_tip_hash(&self) -> Result<BlockHash, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/blocks/tip/hash", BASE_WALLET_API_V1,));

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
            .to_json::<GetTipHashResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(BlockHash::from_str(&parsed.BlockHash).map_err(|_| Error::DeserializeError)?)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::BlockHash;

    use super::BlockClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_blocks() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let blocks = client.get_blocks(Some(0u32)).await;
        println!("request done: {:?}", blocks);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_header_by_hash() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let header = client
            .get_header_by_hash(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
            )
            .await;
        println!("request done: {:?}", header);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_block_hash() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let block_hash = client.get_block_hash(0u32).await;
        println!("request done: {:?}", block_hash);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_block_status() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let block_status = client
            .get_block_status(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
            )
            .await;
        println!("request done: {:?}", block_status);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_block_by_hash() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let block = client
            .get_block_by_hash(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
            )
            .await;
        println!("request done: {:?}", block);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_txid_at_block_index() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let txid = client
            .get_txid_at_block_index(
                &BlockHash::from_str("00000000b873e79784647a6c82962c70d228557d24a747ea4d1b8bbe878e1206").unwrap(),
                0,
            )
            .await;
        println!("request done: {:?}", txid);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_tip_height() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let header = client.get_tip_height().await;
        println!("request done: {:?}", header);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_tip_hash() {
        let session = common_session().await;
        let client = BlockClient::new(session);

        let block_hash = client.get_tip_hash().await;
        println!("request done: {:?}", block_hash);
    }
}
