use std::sync::Arc;

use andromeda_common::{async_trait_impl, Network, ScriptType};
use async_std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use bdk_wallet::{descriptor, KeychainKind, PersistedWallet, Wallet};
use bitcoin::{NetworkKind, PrivateKey};

use crate::{account_trait::AccessWallet, bdk_wallet_secp_ext::BdkWalletSecpExt, error::Error, storage::WalletStorage};

/// A representation of a paper wallet account
#[derive(Debug, Clone)]
pub struct PaperAccount {
    wallet: Arc<RwLock<PersistedWallet<WalletStorage>>>,
    persister: Arc<RwLock<WalletStorage>>,
}

async_trait_impl! {
impl AccessWallet for PaperAccount {

    /// Returns a mutable lock to the account's BdkWallet struct.
    async fn lock_wallet_mut(&self) -> RwLockWriteGuard<PersistedWallet<WalletStorage>> {
        self.wallet.write().await
    }

    /// Returns a readable lock to account's BdkWallet struct
    async fn lock_wallet(&self) -> RwLockReadGuard<PersistedWallet<WalletStorage>> {
        self.wallet.read().await
    }

    /// Returns a mutable lock to the account's BdkWallet struct.
    async fn lock_persister_mut(&self) -> RwLockWriteGuard<WalletStorage> {
        self.persister.write().await
    }
}}

impl PaperAccount {
    /// Creates a new `PaperAccount` from a private key and script type
    fn new(
        script_type: ScriptType,
        priv_key: PrivateKey,
        network: Network,
        storage: Option<WalletStorage>,
    ) -> Result<PaperAccount, Error> {
        // P2wsh unsupported
        let descriptor = match script_type {
            ScriptType::Legacy => descriptor!(pkh(priv_key)),
            ScriptType::NestedSegwit => descriptor!(sh(wpkh(priv_key))),
            ScriptType::NativeSegwit => descriptor!(wpkh(priv_key)),
            ScriptType::Taproot => descriptor!(tr(priv_key)),
        }?;

        let params = Wallet::create_single(descriptor);
        let mut persister = storage.unwrap_or(WalletStorage::memory_persist());
        let paper_wallet = params
            .network(network.into())
            .create_wallet(&mut persister)
            .map_err(|e| Error::CreateWithPersistError(e.to_string()))?;
        Ok(Self {
            wallet: Arc::new(RwLock::new(paper_wallet)),
            persister: Arc::new(RwLock::new(persister)),
        })
    }

    /// Generates a new paper wallet with a random private key
    pub fn generate(network: Network, script_type: ScriptType) -> Result<Self, Error> {
        // 1) Generate a random private key securely
        let btc_network: bitcoin::Network = network.into();
        let priv_key = PrivateKey::generate(btc_network);
        Self::new(script_type, priv_key, network, None)
    }

    /// Creates a `PaperAccount` from an existing WIF private key
    pub fn new_from(
        wif: &str,
        script_type: ScriptType,
        network: Option<Network>,
        storage: Option<WalletStorage>,
    ) -> Result<Self, Error> {
        let priv_key = PrivateKey::from_wif(&wif).map_err(|_e| Error::InvalidWalletWIF)?;
        let network = network.unwrap_or(match priv_key.network {
            NetworkKind::Main => Network::Bitcoin,
            NetworkKind::Test => Network::Regtest,
        });
        Self::new(script_type, priv_key, network, storage)
    }

    /// Retrieves the WIF private key from the wallet
    pub async fn get_wif(&self) -> Result<String, Error> {
        let wallet = self.lock_wallet().await;
        Ok(wallet.get_wif_string()?)
    }

    /// Retrieves the first address from the paper wallet
    pub async fn get_wif_address(&self) -> String {
        let wallet = self.lock_wallet().await;
        wallet.peek_address(KeychainKind::External, 0).address.to_string()
    }
}

#[cfg(test)]
mod tests {

    use std::{str::FromStr, sync::Arc};

    use andromeda_api::{
        tests::utils::{common_api_client, setup_test_connection},
        BASE_WALLET_API_V1,
    };
    use andromeda_common::{Network, ScriptType};
    use bitcoin::{Address, AddressType};
    use wiremock::{
        matchers::{body_string_contains, method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    use super::PaperAccount;
    use crate::{
        account_syncer::AccountSyncer,
        account_trait::AccessWallet,
        blockchain_client::BlockchainClient,
        read_mock_file,
        storage::{WalletFilePersisterFactory, WalletPersisterFactory, WalletStorage},
    };

    fn set_test_account() -> PaperAccount {
        PaperAccount::new_from(
            "Kywx3GGC2d5UDiVT8FEtCMPGbXWJAuD7LUqGdejPYW9bPrRLhr3p",
            ScriptType::NativeSegwit,
            None,
            None,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn test_get_wallet_balance() {
        let account = set_test_account();
        let wallet = account.lock_wallet().await;
        assert!(wallet.balance().total().to_sat() == 0);
    }

    #[tokio::test]
    async fn test_generate() {
        let wallet = PaperAccount::generate(Network::Bitcoin, ScriptType::Legacy).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Bitcoin.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2pkh);

        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let wallet = PaperAccount::generate(Network::Bitcoin, ScriptType::NestedSegwit).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Bitcoin.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2sh);

        let wallet = PaperAccount::generate(Network::Bitcoin, ScriptType::NativeSegwit).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Bitcoin.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2wpkh);

        let wallet = PaperAccount::generate(Network::Bitcoin, ScriptType::Taproot).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Bitcoin.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2tr);

        // testnet
        let wallet = PaperAccount::generate(Network::Testnet, ScriptType::Legacy).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Testnet.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2pkh);

        let wallet = PaperAccount::generate(Network::Testnet, ScriptType::NestedSegwit).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Testnet.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2sh);

        let wallet = PaperAccount::generate(Network::Testnet, ScriptType::NativeSegwit).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Testnet.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2wpkh);

        let wallet = PaperAccount::generate(Network::Testnet, ScriptType::Taproot).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Testnet.into()));
        assert_eq!(
            address.clone().assume_checked().address_type().unwrap(),
            AddressType::P2tr
        );
        assert!(wallet.owns(&address.assume_checked()).await);
    }

    #[tokio::test]
    async fn test_generate_import() {
        let wallet = PaperAccount::generate(Network::Testnet, ScriptType::NativeSegwit).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Testnet.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2wpkh);
        let wif_wallet = PaperAccount::new_from(&wif, ScriptType::NativeSegwit, None, None);
        assert!(wif_wallet.is_ok());

        let wallet = PaperAccount::generate(Network::Bitcoin, ScriptType::NativeSegwit).unwrap();
        let wif = wallet.get_wif().await.unwrap();
        let wif_addr = wallet.get_wif_address().await;
        assert!(!wif.is_empty() && !wif_addr.is_empty());
        let address = Address::from_str(&wif_addr).unwrap();
        assert!(address.is_valid_for_network(Network::Bitcoin.into()));
        assert_eq!(address.assume_checked().address_type().unwrap(), AddressType::P2wpkh);
        let wif_wallet = PaperAccount::new_from(&wif, ScriptType::NativeSegwit, Some(Network::Regtest), None);
        assert!(wif_wallet.is_err());

        let wif_wallet = PaperAccount::new_from(
            "1bKywx3a11GC2d5UDiVT8FEtCMPGbXWJAuD7LUqGdejPYW9bPrRLhr3p",
            ScriptType::NativeSegwit,
            Some(Network::Regtest),
            None,
        );
        assert!(wif_wallet.is_err());
    }

    #[tokio::test]
    #[ignore]
    async fn test_dump_test_wallet() {
        let wif = "cUMvU44fK2P5Kpgd7VnCh6DAidLMzHLjNU9bxqiUXhHCfFS82Fum";
        // let wif_addr = "bcrt1qdq59uwh5azxfpwvcaajwt0pqcwje75me8tpgje";
        let factory = WalletFilePersisterFactory(true);
        let persister = factory.build("regtest_native_segwit_wif_cUMvU44fK2".to_string());
        let account = Arc::new(
            PaperAccount::new_from(&wif, ScriptType::NativeSegwit, None, Some(WalletStorage(persister))).unwrap(),
        );
        let api_client = common_api_client().await;
        let client = Arc::new(BlockchainClient::new(api_client));
        let wallet_sync = AccountSyncer::new(client, account);
        wallet_sync.full_sync(None).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_balance() {
        let account = Arc::new(set_test_account());

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
        let client = Arc::new(BlockchainClient::new(api_client));
        let sync = AccountSyncer::new(client, account);

        // do full sync
        _ = sync.full_sync(None).await;
    }
}
