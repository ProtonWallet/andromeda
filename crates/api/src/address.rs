use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use super::BASE_WALLET_API_V1;
use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    transaction::ApiTransactionStatus,
    ProtonWalletApiClient,
};

pub struct AddressClient {
    api_client: Arc<ProtonWalletApiClient>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct AddressBalance {
    pub Address: String,
    pub ChainFundedBitcoin: u64,
    pub ChainSpentBitcoin: u64,
    pub MempoolFundedBitcoin: u64,
    pub MempoolSpentBitcoin: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetAddressBalanceResponseBody {
    pub Code: u16,
    pub Balance: AddressBalance,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiVout {
    pub ScriptPubKey: String,
    pub ScriptPubKeyAsm: String,
    pub ScriptPubKeyType: String,
    pub ScriptPubKeyAddress: String,
    pub Value: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiVin {
    pub TransactionID: String,
    pub Vout: u32,
    pub Prevout: ApiVout,
    pub ScriptSig: String,
    pub ScriptSigAsm: String,
    pub Witness: Vec<String>,
    pub InnerWitnessScriptAsm: Option<String>,
    pub IsCoinbase: u8,
    pub Sequence: u32,
    pub InnerRedeemScriptAsm: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiTx {
    pub TransactionID: String,
    pub Version: i32,
    pub Locktime: u32,
    pub Vin: Option<Vec<ApiVin>>,
    pub Vout: Option<Vec<ApiVout>>,
    pub Size: u32,
    pub Weight: u32,
    pub Fee: u64,
    pub TransactionStatus: ApiTransactionStatus,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetScriptHashTransactionsResponseBody {
    pub Code: u16,
    pub Transactions: Vec<ApiTx>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetScriptHashTransactionsAtTransactionIDResponseBody {
    pub Code: u16,
    pub Transactions: Vec<ApiTx>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ScriptHashTransactionsPayload {
    pub ScriptHash: String,
    pub TransactionID: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct GetScriptHashesTransactionsRequestBody {
    pub ScriptHashes: Vec<ScriptHashTransactionsPayload>,
}

pub type TransactionsByScriptHash = HashMap<String, Vec<ApiTx>>;

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetScriptHashesTransactionsResponseBody {
    pub Code: u16,
    pub Transactions: TransactionsByScriptHash,
}

impl ApiClient for AddressClient {
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

impl AddressClient {
    /// Get balance of a Bitcoin address.
    pub async fn get_address_balance(&self, address: String) -> Result<AddressBalance, Error> {
        let request = self.get(format!("addresses/{}/balance", address));
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetAddressBalanceResponseBody>()?;

        Ok(parsed.Balance)
    }

    /// Get transaction history for the specified scripthash, sorted by newest
    /// first. Returns up to 50 mempool transactions plus the first 25
    /// confirmed transactions.
    pub async fn get_scripthash_transactions(&self, script_hash: String) -> Result<Vec<ApiTx>, Error> {
        let request = self.get(format!("addresses/scripthash/{}/transactions", script_hash));
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetScriptHashTransactionsResponseBody>()?;

        Ok(parsed.Transactions)
    }

    /// Get transaction history for the specified scripthash, sorted by newest
    /// first. Returns up to 50 mempool transactions plus the first 25
    /// confirmed transactions at TxID.
    pub async fn get_scripthash_transactions_at_transaction_id(
        &self,
        script_hash: String,
        transaction_id: String,
    ) -> Result<Vec<ApiTx>, Error> {
        let request = self.get(format!(
            "addresses/scripthash/{}/transactions/{}",
            script_hash, transaction_id
        ));
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetScriptHashTransactionsResponseBody>()?;

        Ok(parsed.Transactions)
    }

    /// Get transaction history for multiple scripthashes, sorted by newest
    /// first. Returns up to 50 mempool transactions plus the first 25
    /// confirmed transactions. Pass TxID in the structure to get more
    /// transactions at TxID.
    pub async fn get_scripthashes_transactions(
        &self,
        script_hashes: Vec<ScriptHashTransactionsPayload>,
    ) -> Result<TransactionsByScriptHash, Error> {
        let payload = GetScriptHashesTransactionsRequestBody {
            ScriptHashes: script_hashes,
        };

        let request = self.post("addresses/scripthashes/transactions").body_json(payload)?;
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetScriptHashesTransactionsResponseBody>()?;

        Ok(parsed.Transactions)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::{
        hashes::{sha256, Hash},
        Address,
    };
    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::{AddressClient, ScriptHashTransactionsPayload};
    use crate::{
        core::ApiClient,
        tests::utils::{common_api_client, setup_test_connection_arc},
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_address_balance() {
        let api_client = common_api_client().await;
        let client = AddressClient::new(api_client);

        let address = client
            .get_address_balance(String::from("tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6"))
            .await;

        println!("request done: {:?}", address);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_scripthash_transactions() {
        let api_client = common_api_client().await;
        let client = AddressClient::new(api_client);

        let scripthash = sha256::Hash::hash(
            Address::from_str("tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6")
                .unwrap()
                .assume_checked()
                .script_pubkey()
                .as_bytes(),
        );

        let blocks = client.get_scripthash_transactions(scripthash.to_string()).await;

        println!("request done: {:?}", blocks);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_scripthash_transactions_at_transaction_id() {
        let api_client = common_api_client().await;
        let client = AddressClient::new(api_client);

        let scripthash = sha256::Hash::hash(
            Address::from_str("tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6")
                .unwrap()
                .assume_checked()
                .script_pubkey()
                .as_bytes(),
        );

        let blocks = client
            .get_scripthash_transactions_at_transaction_id(
                scripthash.to_string(),
                String::from("ed33ee58f14efbd8a2d9fb6e30bde660c07512dbc7b4d80e00975d3e3d16a302"),
            )
            .await;

        println!("request done: {:?}", blocks);
    }

    #[tokio::test]
    async fn test_get_scripthashes_transactions() {
        let mock_server = MockServer::start().await;

        let script_hash_1 = sha256::Hash::hash(
            Address::from_str("tb1q886jdswcmtn5u9memdlaz0lymua637a9aufqq6")
                .unwrap()
                .assume_checked()
                .script_pubkey()
                .as_bytes(),
        );

        let script_hash_2 = sha256::Hash::hash(
            Address::from_str("tb1q3wp2q7xnsyxlcwtk9vnnwmgpuw0ftlmertppsu")
                .unwrap()
                .assume_checked()
                .script_pubkey()
                .as_bytes(),
        );

        let mut script_hashes = Vec::new();
        script_hashes.push(ScriptHashTransactionsPayload {
            ScriptHash: script_hash_1.to_string(),
            TransactionID: None,
        });
        script_hashes.push(ScriptHashTransactionsPayload {
            ScriptHash: script_hash_2.to_string(),
            TransactionID: Some(String::from(
                "3865f0eb3f59dfbef5506e0269430c773df0fa288bc89fb4fcc99ae1a2d8835e",
            )),
        });

        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "Transactions": {
                script_hash_1.to_string(): [],
                script_hash_2.to_string(): [],
            },
        });

        let req_path: String = format!("{}/addresses/scripthashes/transactions", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "ScriptHashes": [
                    {
                        "ScriptHash": script_hash_1.to_string(),
                        "TransactionID": null,
                    },
                    {
                        "ScriptHash": script_hash_2.to_string(),
                        "TransactionID": "3865f0eb3f59dfbef5506e0269430c773df0fa288bc89fb4fcc99ae1a2d8835e",
                    },
                ],
            })))
            .respond_with(response)
            .expect(1..)
            .with_priority(1)
            .mount(&mock_server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(404))
            .with_priority(2)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection_arc(mock_server.uri());
        let address_client = AddressClient::new(api_client);
        let transactions = address_client.get_scripthashes_transactions(script_hashes).await;
        println!("request done: {:?}", transactions);
        match transactions {
            Ok(value) => {
                assert!(!value.is_empty());
                assert_eq!(value.get(&script_hash_1.to_string()).unwrap().len(), 0);
                assert_eq!(value.get(&script_hash_2.to_string()).unwrap().len(), 0);
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e),
        }

        let unmatched_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(unmatched_requests.len(), 1, "There should be no unmatched requests");
    }
}
