use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};

use super::BASE_WALLET_API_V1;
use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    transaction::ApiTransactionStatus,
    ProtonWalletApiClient,
};

#[derive(Clone)]
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

#[derive(Clone, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiVout {
    pub ScriptPubKey: String,
    pub ScriptPubKeyAsm: String,
    pub ScriptPubKeyType: String,
    pub ScriptPubKeyAddress: Option<String>,
    pub Value: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiVin {
    pub TransactionID: String,
    pub Vout: u32,
    pub Prevout: Option<ApiVout>,
    pub ScriptSig: String,
    pub ScriptSigAsm: String,
    pub Witness: Option<Vec<String>>,
    pub InnerWitnessScriptAsm: Option<String>,
    pub IsCoinbase: u8,
    pub Sequence: u32,
    pub InnerRedeemScriptAsm: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
              &script_hash_1.to_string(): [
                {
                  "TransactionID": "4864cd31446b99ddf378d5166fc66ab1672698f028e68968d004db0b13839ad7",
                  "Version": 1,
                  "Locktime": 2570572,
                  "Vin": [
                    {
                      "TransactionID": "227bef57992d17b9777df39f12ac3ff27393c19d299e8aa5c8eea3131a862c62",
                      "Vout": 0,
                      "Prevout": {
                        "ScriptPubKey": "0014826f689846b5bdbd45859ddb32bd809d196a56e9",
                        "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 826f689846b5bdbd45859ddb32bd809d196a56e9",
                        "ScriptPubKeyType": "v0_p2wpkh",
                        "ScriptPubKeyAddress": "tb1qsfhk3xzxkk7m63v9nhdn90vqn5vk54hff0lrq5",
                        "Value": 8000
                      },
                      "ScriptSig": "",
                      "ScriptSigAsm": "",
                      "Witness": [
                        "3044022041d98ade17d1fe61a8195f0ddf3382f8331359084543bf14f11c2a5b1e0f849a02201b58c49d7173ec81eca8cc856f9c8c23a392b25a097f4cf4b53f13fc96cea80501",
                        "03d06ee6eedcbc8685a4ad87f1b62505d7828bd824c404b2b3ac13cc7668fff535"
                      ],
                      "InnerWitnessScriptAsm": null,
                      "IsCoinbase": 0,
                      "Sequence": 4294967294u64,
                      "InnerRedeemScriptAsm": null
                    }
                  ],
                  "Vout": [
                    {
                      "ScriptPubKey": "0014ff62ea61182b39397d2617d274f4c9bf4131220f",
                      "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 ff62ea61182b39397d2617d274f4c9bf4131220f",
                      "ScriptPubKeyType": "v0_p2wpkh",
                      "ScriptPubKeyAddress": "tb1qla3w5cgc9vunjlfxzlf8faxfhaqnzgs0grpj8h",
                      "Value": 459
                    },
                    {
                      "ScriptPubKey": "0014caf7fc57f4f24cd43ee21adcd725a8cb6fea6b5a",
                      "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 caf7fc57f4f24cd43ee21adcd725a8cb6fea6b5a",
                      "ScriptPubKeyType": "v0_p2wpkh",
                      "ScriptPubKeyAddress": "tb1qetmlc4l57fxdg0hzrtwdwfdgedh7566652ecmf",
                      "Value": 7400
                    }
                  ],
                  "Size": 222,
                  "Weight": 561,
                  "Fee": 141,
                  "TransactionStatus": {
                    "IsConfirmed": 1,
                    "BlockHeight": 2570576,
                    "BlockHash": "00000000000000100898f2f1121229ae689e27b2d1f0970f015ee27ae81c9aff",
                    "BlockTime": 1704358819
                  }
                },
                {
                  "TransactionID": "227bef57992d17b9777df39f12ac3ff27393c19d299e8aa5c8eea3131a862c62",
                  "Version": 2,
                  "Locktime": 2545684,
                  "Vin": [
                    {
                      "TransactionID": "f1659208d528a1c4eaa9223667a3b66b56230622e48fe92236a93ab6f9df3bef",
                      "Vout": 0,
                      "Prevout": {
                        "ScriptPubKey": "0014803a7c4f9b0ee47a3da69b52ac19f580c348bbd4",
                        "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 803a7c4f9b0ee47a3da69b52ac19f580c348bbd4",
                        "ScriptPubKeyType": "v0_p2wpkh",
                        "ScriptPubKeyAddress": "tb1qsqa8cnumpmj850dxndf2cx04srp53w75y0n9jw",
                        "Value": 209280
                      },
                      "ScriptSig": "",
                      "ScriptSigAsm": "",
                      "Witness": [
                        "304402201f824767de76c12098f0b04b0692fbb66c26524fb5241ab2b6deff60478d91a602207323df0292bb3eb45423519b0058fce4dfa4c45d7e08f3b9cae7ff91e1f17b8501",
                        "03ff93f204b5c827eb76252394c749cd68265882ba94384394538512cb85c56658"
                      ],
                      "InnerWitnessScriptAsm": null,
                      "IsCoinbase": 0,
                      "Sequence": 4294967293u64,
                      "InnerRedeemScriptAsm": null
                    }
                  ],
                  "Vout": [
                    {
                      "ScriptPubKey": "0014826f689846b5bdbd45859ddb32bd809d196a56e9",
                      "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 826f689846b5bdbd45859ddb32bd809d196a56e9",
                      "ScriptPubKeyType": "v0_p2wpkh",
                      "ScriptPubKeyAddress": "tb1qsfhk3xzxkk7m63v9nhdn90vqn5vk54hff0lrq5",
                      "Value": 8000
                    },
                    {
                      "ScriptPubKey": "0014267a528b872a04d53d3d83fea90da64e2605a1a2",
                      "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 267a528b872a04d53d3d83fea90da64e2605a1a2",
                      "ScriptPubKeyType": "v0_p2wpkh",
                      "ScriptPubKeyAddress": "tb1qyea99zu89gzd20fas0l2jrdxfcnqtgdz6z9ndq",
                      "Value": 200997
                    }
                  ],
                  "Size": 222,
                  "Weight": 561,
                  "Fee": 283,
                  "TransactionStatus": {
                    "IsConfirmed": 1,
                    "BlockHeight": 2545697,
                    "BlockHash": "000000000007a75255087dabacbef5ccbc2828a039d71bac6b8d7735243b5f50",
                    "BlockTime": 1703574011
                  }
                }
              ],
              &script_hash_2.to_string(): [
                {
                  "TransactionID": "7de0a575f912e6a08bc9e9a22a5c4d86318c22c23298a02373ff5dffddb8307d",
                  "Version": 1,
                  "Locktime": 2570572,
                  "Vin": [
                    {
                      "TransactionID": "6a7fb0885a0afd6e9092a7eac2cb77dc65e46419efacb699480d850fded88c3d",
                      "Vout": 0,
                      "Prevout": {
                        "ScriptPubKey": "0014749fe0c1ad3c4fb90fea4aa5d4a1dff8d2b234a4",
                        "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 749fe0c1ad3c4fb90fea4aa5d4a1dff8d2b234a4",
                        "ScriptPubKeyType": "v0_p2wpkh",
                        "ScriptPubKeyAddress": "tb1qwj07psdd838mjrl2f2jafgwllrftyd9yjug4ps",
                        "Value": 2059
                      },
                      "ScriptSig": "",
                      "ScriptSigAsm": "",
                      "Witness": [
                        "3044022051c5438f66054f3e37bcf7a7e12aa5cc33c4472ea2f942d9ccab5d3a4e60a72c0220446cde78c0ac2bc55596421c98c05e9b09902223250b58da003809d296ba157301",
                        "036593beafc40cb6d886def43569ffd91d8e21954d94d0e219dda1bea09be508a6"
                      ],
                      "InnerWitnessScriptAsm": null,
                      "IsCoinbase": 0,
                      "Sequence": 4294967294u64,
                      "InnerRedeemScriptAsm": null
                    }
                  ],
                  "Vout": [
                    {
                      "ScriptPubKey": "0014db96479b24ff8b6644318e9b45e689d60e00fea2",
                      "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 db96479b24ff8b6644318e9b45e689d60e00fea2",
                      "ScriptPubKeyType": "v0_p2wpkh",
                      "ScriptPubKeyAddress": "tb1qmwty0xeyl79kv3p336d5te5f6c8qpl4zm7zefu",
                      "Value": 790
                    },
                    {
                      "ScriptPubKey": "0014b8bd3c495fc429ac304f7677e8dccbdbf0404399",
                      "ScriptPubKeyAsm": "OP_0 OP_PUSHBYTES_20 b8bd3c495fc429ac304f7677e8dccbdbf0404399",
                      "ScriptPubKeyType": "v0_p2wpkh",
                      "ScriptPubKeyAddress": "tb1qhz7ncj2lcs56cvz0wem73hxtm0cyqsuem6z5te",
                      "Value": 1128
                    }
                  ],
                  "Size": 222,
                  "Weight": 561,
                  "Fee": 141,
                  "TransactionStatus": {
                    "IsConfirmed": 1,
                    "BlockHeight": 2570575,
                    "BlockHash": "0000000000000033658b475be1c95cb213f987dc7dcdf9d05bcb48de71400003",
                    "BlockTime": 1704358197
                  }
                }
              ]
            }
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
                assert!(!value.get(&script_hash_1.to_string()).unwrap().is_empty());
                assert!(!value.get(&script_hash_2.to_string()).unwrap().is_empty());
            }
            Err(e) => panic!("Expected Ok variant but got Err.{}", e),
        }

        let unmatched_requests = mock_server.received_requests().await.unwrap();
        assert_eq!(unmatched_requests.len(), 1, "There should be no unmatched requests");
    }
}
