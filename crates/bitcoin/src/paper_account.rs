use std::sync::Arc;

use andromeda_common::async_trait_impl;
use async_std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use bdk_wallet::{bitcoin::Address, PersistedWallet, Wallet};
use bitcoin::{AddressType, Network, NetworkKind, PrivateKey};

use crate::{
    account_trait::AccessWallet,
    error::Error,
    storage::{MemoryPersisted, WalletStorage},
};

#[derive(Debug, Clone)]
pub struct PaperAccount {
    wallet: Arc<RwLock<PersistedWallet<WalletStorage>>>,
}

async_trait_impl! {
impl AccessWallet for PaperAccount {
    async fn get_mutable_wallet(&self) -> RwLockWriteGuard<PersistedWallet<WalletStorage>> {
        self.wallet.write().await
    }

    /// Returns a readable lock to account's BdkWallet struct
    async fn get_wallet(&self) -> RwLockReadGuard<PersistedWallet<WalletStorage>> {
        self.wallet.read().await
    }
}}

/// Notes:: this is not ready for production, it is just for a experimental purpose
impl PaperAccount {
    pub fn new(wif: &str, address_type: AddressType) -> Result<Self, Error> {
        let descriptor = match address_type {
            AddressType::P2pkh => format!("pkh({})", wif),
            AddressType::P2wsh => format!("wsh({})", wif),
            AddressType::P2sh => format!("sh({})", wif),
            AddressType::P2wpkh => format!("wpkh({})", wif),
            AddressType::P2tr => format!("tr({})", wif),
            _ => return Err(Error::InvalidScriptType),
        };

        let private_key = PrivateKey::from_wif(&wif);

        let params = Wallet::create_single(descriptor.to_string());

        let network = match private_key.map(|key| key.network) {
            Ok(NetworkKind::Main) => Network::Bitcoin,
            Ok(NetworkKind::Test) => Network::Regtest,
            _ => return Err(Error::InvalidNetwork),
        };
        let mut persiste = WalletStorage(Arc::new(MemoryPersisted {}));
        let paper_wallet = params
            .network(network)
            .create_wallet(&mut persiste)
            .map_err(|_e| Error::CreateWithPersistError)?;

        Ok(Self {
            wallet: Arc::new(RwLock::new(paper_wallet)),
        })
    }

    /// Returns a boolean indicating whether or not the account owns the
    /// provided address
    pub async fn owns(&self, address: &Address) -> bool {
        self.get_wallet().await.is_mine(address.script_pubkey())
    }
}

#[cfg(test)]
mod tests {

    use andromeda_api::{tests::utils::setup_test_connection, BASE_WALLET_API_V1};
    use bitcoin::AddressType;
    use wiremock::{
        matchers::{body_string_contains, method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    use super::PaperAccount;
    use crate::{account_trait::AccessWallet, blockchain_client::BlockchainClient, read_mock_file};

    fn set_test_account() -> PaperAccount {
        PaperAccount::new(
            "Kywx3GGC2d5UDiVT8FEtCMPGbXWJAuD7LUqGdejPYW9bPrRLhr3p",
            AddressType::P2wpkh,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_get_wallet() {
        let account = set_test_account();
        let wallet = account.get_wallet().await;
        assert!(wallet.balance().total().to_sat() == 0);
    }

    #[tokio::test]
    async fn test_get_balance() {
        let account = set_test_account();
        let mock_server = MockServer::start().await;
        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);
        let response_contents = read_mock_file!("get_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);

        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_1");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;

        let response_contents2 = read_mock_file!("get_scripthashes_transactions_body_2");
        let response2 = ResponseTemplate::new(200).set_body_string(response_contents2);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "b6c3616a787f87ed96b70770d84d45acf637ed3ad6f2706b2dfc282cc3ba4c05",
            ))
            .respond_with(response2)
            .mount(&mock_server)
            .await;

        let response_contents3 = read_mock_file!("get_scripthashes_transactions_body_3");
        let response3 = ResponseTemplate::new(200).set_body_string(response_contents3);

        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "5eac955f250ff14fd8c61e29e9531bc3e49d69038981a1344e88b985bd200a29",
            ))
            .respond_with(response3)
            .mount(&mock_server)
            .await;

        let response_contents_block_hash = read_mock_file!("get_block_hash_body");
        let response_block_hash = ResponseTemplate::new(200).set_body_string(response_contents_block_hash);

        Mock::given(method("GET"))
            .and(path_regex(".*/height/.*"))
            .respond_with(response_block_hash)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = BlockchainClient::new(api_client.clone());

        // do full sync
        _ = client.full_sync(&account, None).await;
    }
}
