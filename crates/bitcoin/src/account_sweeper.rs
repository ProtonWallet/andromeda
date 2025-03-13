use std::sync::Arc;

use andromeda_common::ScriptType;
use bdk_wallet::{AddressInfo, KeychainKind, SignOptions};
use bitcoin::FeeRate;

use crate::{
    account_syncer::AccountSyncer,
    account_trait::{AccessWallet, AccessWalletDyn},
    blockchain_client::BlockchainClient,
    error::Error,
    paper_account::PaperAccount,
    psbt::Psbt,
};

#[derive(Clone)]
pub struct AccountSweeper {
    client: Arc<BlockchainClient>,
    account: Arc<AccessWalletDyn>,
}

impl AccountSweeper {
    pub fn new(client: Arc<BlockchainClient>, account: Arc<AccessWalletDyn>) -> Self {
        Self { client, account }
    }
}

impl AccountSweeper {
    /// Sweep BTC from a paper wallet into the user's wallet
    pub async fn get_sweep_psbt(
        &self,
        account: Arc<PaperAccount>,
        sat_per_vb: u64,
        address: AddressInfo,
    ) -> Result<Psbt, Error> {
        if !account.has_sync_data().await {
            return Err(Error::WalletNotSynced);
        }
        let mut wallet_lock = account.lock_wallet_mut().await;
        let mut tx_builder = wallet_lock.build_tx();
        tx_builder.drain_to(address.script_pubkey()).drain_wallet();
        tx_builder.fee_rate(FeeRate::from_sat_per_vb(sat_per_vb).ok_or(Error::InvalidNetwork)?);
        let mut psbt = tx_builder.finish()?;
        wallet_lock.sign(&mut psbt, SignOptions::default())?;
        Ok(Psbt::from(psbt))
    }

    /// Sweep BTC from a WIF private key and its corresponding address into the user's wallet
    pub async fn get_sweep_wif_psbt(
        &self,
        wif: &str,
        sat_per_vb: u64,
        receive_address_index: Option<u32>,
    ) -> Result<(Psbt, String), Error> {
        let recipient_address = {
            let mut wallet = self.account.lock_wallet_mut().await;
            match receive_address_index {
                Some(index) => wallet.peek_address(KeychainKind::External, index),
                None => wallet.next_unused_address(KeychainKind::External),
            }
        };

        let script_types = ScriptType::values();
        let network = { self.account.lock_wallet().await.network() };
        let mut found_wallet = false;
        for address_type in script_types {
            let paper_account = match PaperAccount::new_from(wif, address_type, Some(network.into()), None) {
                Ok(wallet) => Arc::new(wallet),
                Err(_) => continue,
            };

            // Sync wallet
            let wallet_sync = AccountSyncer::new(self.client.clone(), paper_account.clone());
            if !wallet_sync.full_sync(None).await.is_ok() {
                continue;
            }

            if let Ok(psbt) = self
                .get_sweep_psbt(paper_account.clone(), sat_per_vb, recipient_address.clone())
                .await
            {
                // Only return if there are actual funds to sweep
                if psbt.outputs_amount()?.to_sat() > 0 {
                    return Ok((psbt, paper_account.get_wif_address().await));
                }
            }
            found_wallet = true;
        }

        if found_wallet {
            return Err(Error::InsufficientFundsInPaperWallet);
        }

        Err(Error::InvalidPaperWallet)
    }

    // Sweep BTC from the user's wallet into a paper wallet (specified by WIF)
    // pub async fn psbt_sweep_to(&self, wif: &str, sat_per_vb: u64, receive_address_index: Option<u32>) {}
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use andromeda_api::{tests::utils::setup_test_connection, BASE_WALLET_API_V1};
    use andromeda_common::{Network, ScriptType};
    use wiremock::{
        matchers::{body_string_contains, method, path, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        account_sweeper::AccountSweeper, blockchain_client::BlockchainClient, error::Error, read_mock_file,
        tests::utils::tests::set_test_wallet_account,
    };

    #[tokio::test]
    async fn test_sweep_wallet_with_wif() {
        let wif = "cUMvU44fK2P5Kpgd7VnCh6DAidLMzHLjNU9bxqiUXhHCfFS82Fum";
        let wif_addr = "bcrt1qdq59uwh5azxfpwvcaajwt0pqcwje75me8tpgje";

        let account = Arc::new(set_test_wallet_account(
            "onion ancient develop team busy purchase salmon robust danger wheat rich empower",
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            None,
            None,
            Some(Network::Regtest),
            None,
        ));

        let mock_server = MockServer::start().await;
        let req_path_blocks: String = format!("{}/blocks", BASE_WALLET_API_V1);
        let response_contents = read_mock_file!("get_wif_blocks_body");
        let response = ResponseTemplate::new(200).set_body_string(response_contents);
        Mock::given(method("GET"))
            .and(path(req_path_blocks.clone()))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);
        let response_contents1 = read_mock_file!("get_wif_scripthashes_transactions");
        let response1 = ResponseTemplate::new(200).set_body_string(response_contents1);
        Mock::given(method("POST"))
            .and(path(req_path.clone()))
            .and(body_string_contains(
                "f7267338f0adaa29302d1675467cb7c416110d5359f79b868f9c96178a5f838b",
            ))
            .respond_with(response1)
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

        let sweep = AccountSweeper::new(client, account.clone());
        let psbt = sweep.get_sweep_wif_psbt(&wif, 1, None).await.unwrap();
        assert_eq!(psbt.1, wif_addr);
        let psbt = sweep.get_sweep_wif_psbt(wif, 1, Some(2)).await.unwrap();
        assert_eq!(psbt.1, wif_addr);
        assert!(psbt.0.outputs_amount().unwrap().to_sat() > 0);
        let psbt = sweep.get_sweep_wif_psbt(wif, 10000000000000, Some(1)).await;
        assert!(psbt.is_err());
        assert_eq!(
            psbt.err().unwrap().to_string(),
            Error::InsufficientFundsInPaperWallet.to_string()
        );
    }
}
