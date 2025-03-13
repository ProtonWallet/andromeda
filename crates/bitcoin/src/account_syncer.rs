// WiFWallet/PaperWallet : account_trait

use std::sync::Arc;

use andromeda_esplora::EsploraAsyncExt;
use bdk_wallet::KeychainKind;
use bitcoin::Txid;

use crate::{account_trait::AccessWalletDyn, blockchain_client::BlockchainClient, error::Error};

pub const DEFAULT_STOP_GAP: usize = 50;
pub const PARALLEL_REQUESTS: usize = 5;

/// `AccountSyncer` is responsible for synchronizing a wallet with the blockchain.
#[derive(Clone)]
pub struct AccountSyncer {
    client: Arc<BlockchainClient>,
    account: Arc<AccessWalletDyn>,
}

impl AccountSyncer {
    /// Creates a new instance of `AccountSyncer`.
    pub fn new(client: Arc<BlockchainClient>, account: Arc<AccessWalletDyn>) -> Self {
        Self { client, account }
    }
}

impl AccountSyncer {
    /// Performs a full wallet synchronization up to a defined stop-gap.
    ///
    /// This process scans script pubkeys sequentially, fetching transactions until
    /// a gap of unused addresses is reached. After retrieving transactions, it verifies
    /// block confirmations and tracks unspent outputs.
    ///
    /// **Locks**: Acquires a **read lock** on the wallet for extracting sync requests.  
    ///            The actual update applies a **write lock** inside `apply_update()`.  
    ///
    /// # Arguments
    /// - `stop_gap`: Optional stop-gap value (default: `DEFAULT_STOP_GAP`).
    ///
    /// # Returns
    /// - `Ok(())` if the sync was successful.
    /// - `Err(Error)` if the sync fails.
    pub async fn full_sync(&self, stop_gap: Option<usize>) -> Result<(), Error> {
        let request = {
            // 🔒 Acquire read lock
            let read_lock = self.account.lock_wallet().await;
            // Extract the request while holding the lock
            let request = read_lock.start_full_scan();
            request
        }; // 🔓 Read lock released

        let update = self
            .client
            .inner()
            .full_scan(request, stop_gap.unwrap_or(DEFAULT_STOP_GAP))
            .await?;

        // Applies update with a write lock inside `apply_update()`
        self.account.apply_update(update.into()).await?;
        Ok(())
    }

    /// Performs a **partial synchronization** for tracking:
    /// - Unconfirmed transactions.
    /// - Outpoints spent.
    /// - New transactions received on unused addresses.
    ///
    /// Notes:
    /// - This has to be done on top of a full sync.
    ///
    /// **Locks**: Acquires a **read lock** to extract wallet state for sync.  
    ///            Applies a **write lock** inside `apply_update()`.  
    ///
    /// # Returns
    /// - `Ok(())` if partial sync was successful.
    /// - `Err(Error)` if sync fails.
    pub async fn partial_sync(&self) -> Result<(), Error> {
        let request = {
            // 🔒 Acquire a read lock to access wallet data
            let read_lock = self.account.lock_wallet().await;

            let chain = read_lock.local_chain();
            let chain_tip = chain.tip().block_id();

            let utxos = read_lock.list_unspent().map(|utxo| utxo.outpoint).collect::<Vec<_>>();

            let unconfirmed_txids = read_lock
                .tx_graph()
                .list_canonical_txs(chain, chain_tip)
                .filter(|canonical_tx| !canonical_tx.chain_position.is_confirmed())
                .map(|canonical_tx| canonical_tx.tx_node.txid)
                .collect::<Vec<Txid>>();

            // Create sync request
            read_lock
                .start_sync_with_revealed_spks()
                .outpoints(utxos.into_iter())
                .txids(unconfirmed_txids.into_iter())
        }; // 🔓 Read lock released

        let update = self.client.inner().sync(request, PARALLEL_REQUESTS).await?;

        // Applies update with a write lock inside `apply_update()`
        self.account.apply_update(update.into()).await?;

        Ok(())
    }

    /// Returns whether or not the wallet needs to be synced again (new block)
    pub async fn should_sync(&self) -> Result<bool, Error> {
        let tip_hash = self.client.inner().get_tip_hash().await?;
        let read_lock = self.account.lock_wallet().await;
        let latest_chekpoint_hash = read_lock.latest_checkpoint().hash();
        Ok(tip_hash != latest_chekpoint_hash)
    }

    /// Special minimal sync to check account existence
    pub async fn check_account_existence(&self, stop_gap: usize) -> Result<bool, Error> {
        let spks = {
            // 🔒 Acquire a read lock to access wallet data
            let read_lock = self.account.lock_wallet().await;
            let spks = read_lock.spk_index().all_unbounded_spk_iters();
            let external_keychain_spks = spks.get(&KeychainKind::External);
            external_keychain_spks
                .map(|spks| spks.clone().take(stop_gap).collect::<Vec<_>>())
                .unwrap_or_default()
        }; // 🔓 Read lock is released here

        let results = self.client.inner().many_scripthash_txs(spks).await.ok();
        // Check if any transactions exist
        if let Some(results) = results {
            return Ok(results.values().any(|(_index, txs)| !txs.is_empty()));
        };

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use andromeda_api::{tests::utils::setup_test_connection, BASE_WALLET_API_V1};
    use andromeda_common::ScriptType;
    use bdk_wallet::serde_json;
    use wiremock::{
        matchers::{body_string_contains, method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        account_syncer::{AccountSyncer, DEFAULT_STOP_GAP},
        blockchain_client::BlockchainClient,
        read_mock_file,
        tests::utils::tests::set_test_wallet_account,
    };

    #[tokio::test]
    async fn test_wallet_sync_without_storage() {
        // Test used pre-build mock file mock_regtest_a8cedf51_84__1__0_.json
        let account = Arc::new(set_test_wallet_account(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower",
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(false),
            None,
            None,
            None,
        ));
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
        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_4");
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
        let client = Arc::new(BlockchainClient::new(api_client.clone()));
        let sync = AccountSyncer::new(client, account.clone());

        // do full sync
        sync.full_sync(None).await.unwrap();

        let contents = read_mock_file!("get_block_hash_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let req_path: String = format!("{}/blocks/tip/hash", BASE_WALLET_API_V1,);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        // check
        let should = sync.should_sync().await.unwrap();
        assert!(should);
        // do partial sync
        let txid = "6b62ad31e219c9dab4d7e24a0803b02bbc5d86ba53f6f02aa6de0f301b718e88";
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

        sync.partial_sync().await.unwrap();
    }

    #[tokio::test]
    async fn test_check_account_existence_without_storage() {
        // Test used pre-build mock file mock_regtest_a8cedf51_84__1__0_.json
        let account = Arc::new(set_test_wallet_account(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower",
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(false),
            None,
            None,
            None,
        ));
        let mock_server = MockServer::start().await;
        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);
        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_4");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;
        let api_client: Arc<andromeda_api::ProtonWalletApiClient> = setup_test_connection(mock_server.uri());
        let client = Arc::new(BlockchainClient::new(api_client));
        let sync = AccountSyncer::new(client, account.clone());

        let exsits = sync.check_account_existence(DEFAULT_STOP_GAP).await.unwrap();
        assert!(exsits);
    }

    #[tokio::test]
    async fn test_check_account_existence_no_without_storage() {
        // Test used pre-build mock file mock_regtest_a8cedf51_84__1__0_.json
        let account = Arc::new(set_test_wallet_account(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower",
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(false),
            None,
            None,
            None,
        ));
        let mock_server = MockServer::start().await;

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);
        let response_contents1 = read_mock_file!("get_scripthashes_transactions_body_5");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "89a10f34b9e0ad8b770c381d5bbb1f566124d3164781f41fb98218d1362069ec",
            ))
            .respond_with(response1)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = Arc::new(BlockchainClient::new(api_client));
        let sync = AccountSyncer::new(client, account.clone());
        let exsits = sync.check_account_existence(DEFAULT_STOP_GAP).await.unwrap();
        assert!(!exsits);
    }

    #[tokio::test]
    async fn test_check_account_existence_no_error_without_storage() {
        // Test used pre-build mock file mock_regtest_a8cedf51_84__1__0_.json
        let account = Arc::new(set_test_wallet_account(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower",
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(false),
            None,
            None,
            None,
        ));
        let mock_server = MockServer::start().await;

        let api_client = setup_test_connection(mock_server.uri());
        let client = Arc::new(BlockchainClient::new(api_client));
        let sync = AccountSyncer::new(client, account.clone());
        let exsits = sync.check_account_existence(DEFAULT_STOP_GAP).await.unwrap();
        assert!(!exsits);
    }
}
