use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

const ONLY_WITHOUT_BITCOIN_ADDRESS_KEY: &str = "OnlyWithoutBitcoinAddresses[]";

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletBitcoinAddress {
    pub ID: String,
    pub WalletID: String,
    pub WalletAccountID: String,
    pub Fetched: u8,
    pub Used: u8,
    pub BitcoinAddress: Option<String>,
    pub BitcoinAddressSignature: Option<String>,
    pub BitcoinAddressIndex: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AddBitcoinAddressesRequestBody {
    pub BitcoinAddresses: Vec<ApiBitcoinAddressCreationPayload>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct ApiBitcoinAddressCreationPayload {
    pub BitcoinAddress: String,
    pub BitcoinAddressSignature: String,
    pub BitcoinAddressIndex: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBitcoinAddressHighestIndexResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub HighestIndex: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBitcoinAddressesResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddresses: Vec<ApiWalletBitcoinAddress>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateBitcoinAddressResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddress: ApiWalletBitcoinAddress,
}

#[derive(Clone)]
pub struct BitcoinAddressClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for BitcoinAddressClient {
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

impl BitcoinAddressClient {
    pub async fn get_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        only_without_bitcoin_addresses: Option<u8>,
    ) -> Result<Vec<ApiWalletBitcoinAddress>, Error> {
        let mut request = self.get(format!(
            "wallets/{}/accounts/{}/addresses/bitcoin",
            wallet_id, wallet_account_id
        ));
        if let Some(only_without_bitcoin_addresses) = only_without_bitcoin_addresses {
            request = request.query((
                ONLY_WITHOUT_BITCOIN_ADDRESS_KEY,
                only_without_bitcoin_addresses.to_string(),
            ));
        }
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBitcoinAddressesResponseBody>()?;

        Ok(parsed.WalletBitcoinAddresses)
    }

    pub async fn get_bitcoin_address_highest_index(
        &self,
        wallet_id: String,
        wallet_account_id: String,
    ) -> Result<u64, Error> {
        let request = self.get(format!(
            "wallets/{}/accounts/{}/addresses/bitcoin/index",
            wallet_id, wallet_account_id,
        ));
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBitcoinAddressHighestIndexResponseBody>()?;
        Ok(parsed.HighestIndex)
    }

    pub async fn add_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        bitcoin_addresses: Vec<ApiBitcoinAddressCreationPayload>,
    ) -> Result<Vec<ApiWalletBitcoinAddress>, Error> {
        let payload = AddBitcoinAddressesRequestBody {
            BitcoinAddresses: bitcoin_addresses,
        };

        let request = self
            .post(format!(
                "wallets/{}/accounts/{}/addresses/bitcoin",
                wallet_id, wallet_account_id,
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBitcoinAddressesResponseBody>()?;
        Ok(parsed.WalletBitcoinAddresses)
    }

    pub async fn update_bitcoin_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_account_bitcoin_address_id: String,
        bitcoin_address: ApiBitcoinAddressCreationPayload,
    ) -> Result<ApiWalletBitcoinAddress, Error> {
        let request = self
            .put(format!(
                "wallets/{}/accounts/{}/addresses/bitcoin/{}",
                wallet_id, wallet_account_id, wallet_account_bitcoin_address_id
            ))
            .body_json(bitcoin_address)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateBitcoinAddressResponseBody>()?;
        Ok(parsed.WalletBitcoinAddress)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use wiremock::{
        matchers::{body_json, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        bitcoin_address::{ApiBitcoinAddressCreationPayload, BitcoinAddressClient},
        core::ApiClient,
        tests::utils::setup_test_connection,
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    async fn test_get_get_bitcoin_addresses_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletBitcoinAddresses": [
                    {
                        "ID": "8gdVKE1364EL3g0VvQOZKIlh97RoiDS3CfJqiEyaT4T2V1sWtUV8JmUgm0foaHfvCEjVOuE5MqKOM32mp2QEKg==",
                        "WalletID": "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==",
                        "WalletAccountID": "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==",
                        "Fetched": 1,
                        "Used": 0,
                        "BitcoinAddress": "bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5",
                        "BitcoinAddressSignature": null,
                        "BitcoinAddressIndex": 9
                    },
                    {
                        "ID": "nW0I1UDIiH_-pWHv7UbFoX9lp2MBohiDgz1HBI_mtnkbvNVe_CldWi1WEpfKPflyhwN9uIMF8z-pkn0CyK3lkA==",
                        "WalletID": "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==",
                        "WalletAccountID": "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==",
                        "Fetched": 0,
                        "Used": 0,
                        "BitcoinAddress": "bc1q3msh39t8eycqfpyx85yk3rehluhfjly0elp6q4",
                        "BitcoinAddressSignature": "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYEABYIAFgFgmc0QMEJkJSe7m6ROP3SMJSAAAAAABEAFmNvbnRleHRAcHJv\ndG9uLmNod2FsbGV0LmJpdGNvaW4tYWRkcmVzcxYhBAEIescS5mon5YPMlZSe\n7m6ROP3SAAC7QgD/WSFsK78Hz5K2wkJXkUW+VFd9XnMmJgUHVysQN174mX0B\nAN5jtQlRsUpRi8jMsm30jOgJcxAiJLFFYbs0OsRgLGcL\n=XKm4\n-----END PGP SIGNATURE-----\n",
                        "BitcoinAddressIndex": 10
                    },
                ],
            }
        );
        let wallet_id = "wallet_001";
        let wallet_account_id = "account_001";
        let req_path: String = format!(
            "{}/wallets/{}/accounts/{}/addresses/bitcoin",
            BASE_WALLET_API_V1, wallet_id, wallet_account_id
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BitcoinAddressClient::new(Arc::new(api_client));
        let result = client
            .get_bitcoin_addresses(wallet_id.to_string(), wallet_account_id.to_string(), None)
            .await;
        match result {
            Ok(addresses) => {
                assert_eq!(
                    addresses[0].ID,
                    "8gdVKE1364EL3g0VvQOZKIlh97RoiDS3CfJqiEyaT4T2V1sWtUV8JmUgm0foaHfvCEjVOuE5MqKOM32mp2QEKg=="
                );
                assert_eq!(
                    addresses[0].WalletID,
                    "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg=="
                );
                assert_eq!(
                    addresses[0].WalletAccountID,
                    "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A=="
                );
                assert_eq!(addresses[0].Fetched, 1);
                assert_eq!(addresses[0].Used, 0);
                assert_eq!(
                    addresses[0].BitcoinAddress,
                    Some("bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5".to_string())
                );
                assert!(addresses[0].BitcoinAddressSignature.is_none());
                assert_eq!(addresses[0].BitcoinAddressIndex, Some(9));

                assert_eq!(
                    addresses[1].ID,
                    "nW0I1UDIiH_-pWHv7UbFoX9lp2MBohiDgz1HBI_mtnkbvNVe_CldWi1WEpfKPflyhwN9uIMF8z-pkn0CyK3lkA=="
                );
                assert_eq!(
                    addresses[1].WalletID,
                    "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg=="
                );
                assert_eq!(
                    addresses[1].WalletAccountID,
                    "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A=="
                );
                assert_eq!(addresses[1].Fetched, 0);
                assert_eq!(addresses[1].Used, 0);
                assert_eq!(
                    addresses[1].BitcoinAddress,
                    Some("bc1q3msh39t8eycqfpyx85yk3rehluhfjly0elp6q4".to_string())
                );
                assert_eq!(addresses[1].BitcoinAddressSignature, Some("-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYEABYIAFgFgmc0QMEJkJSe7m6ROP3SMJSAAAAAABEAFmNvbnRleHRAcHJv\ndG9uLmNod2FsbGV0LmJpdGNvaW4tYWRkcmVzcxYhBAEIescS5mon5YPMlZSe\n7m6ROP3SAAC7QgD/WSFsK78Hz5K2wkJXkUW+VFd9XnMmJgUHVysQN174mX0B\nAN5jtQlRsUpRi8jMsm30jOgJcxAiJLFFYbs0OsRgLGcL\n=XKm4\n-----END PGP SIGNATURE-----\n".to_string()));
                assert_eq!(addresses[1].BitcoinAddressIndex, Some(10));
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_bitcoin_address_highest_index_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "HighestIndex": 100,
            }
        );
        let wallet_id = "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==";
        let wallet_account_id =
            "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==";
        let req_path: String = format!(
            "{}/wallets/{}/accounts/{}/addresses/bitcoin/index",
            BASE_WALLET_API_V1, wallet_id, wallet_account_id
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BitcoinAddressClient::new(Arc::new(api_client));
        let result = client
            .get_bitcoin_address_highest_index(wallet_id.to_string(), wallet_account_id.to_string())
            .await;
        match result {
            Ok(highest_index) => {
                assert_eq!(highest_index, 100);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_add_bitcoin_addresses() {
        let bitcoin_addresses: Vec<ApiBitcoinAddressCreationPayload> = vec![ApiBitcoinAddressCreationPayload {
            BitcoinAddress: "bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5".to_string(),
            BitcoinAddressSignature:
                "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYE-----END PGP SIGNATURE-----\n".to_string(),
            BitcoinAddressIndex: 13,
        }];

        let mock_server = MockServer::start().await;
        let wallet_id = "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==";
        let wallet_account_id =
            "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==";
        let req_path: String = format!(
            "{}/wallets/{}/accounts/{}/addresses/bitcoin",
            BASE_WALLET_API_V1, wallet_id, wallet_account_id
        );
        let response_body = serde_json::json!(
        {
            "Code": 1000,
            "WalletBitcoinAddresses": [
                {
                    "ID": "8gdVKE1364EL3g0VvQOZKIlh97RoiDS3CfJqiEyaT4T2V1sWtUV8JmUgm0foaHfvCEjVOuE5MqKOM32mp2QEKg==",
                    "WalletID": "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==",
                    "WalletAccountID": "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==",
                    "Fetched": 0,
                    "Used": 0,
                    "BitcoinAddress": "bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5",
                    "BitcoinAddressSignature": "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYE-----END PGP SIGNATURE-----\n",
                    "BitcoinAddressIndex": 13
                },
            ]
        });
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "BitcoinAddresses": [
                    {
                        "BitcoinAddress": "bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5",
                        "BitcoinAddressSignature": "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYE-----END PGP SIGNATURE-----\n",
                        "BitcoinAddressIndex": 13,
                    }
                ],
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BitcoinAddressClient::new(Arc::new(api_client));
        let result = client
            .add_bitcoin_addresses(wallet_id.to_string(), wallet_account_id.to_string(), bitcoin_addresses)
            .await;
        match result {
            Ok(addresses) => {
                assert_eq!(
                    addresses[0].ID,
                    "8gdVKE1364EL3g0VvQOZKIlh97RoiDS3CfJqiEyaT4T2V1sWtUV8JmUgm0foaHfvCEjVOuE5MqKOM32mp2QEKg=="
                );
                assert_eq!(
                    addresses[0].WalletID,
                    "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg=="
                );
                assert_eq!(
                    addresses[0].WalletAccountID,
                    "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A=="
                );
                assert_eq!(addresses[0].Fetched, 0);
                assert_eq!(addresses[0].Used, 0);
                assert_eq!(
                    addresses[0].BitcoinAddress,
                    Some("bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5".to_string())
                );
                assert_eq!(
                    addresses[0].BitcoinAddressSignature,
                    Some(
                        "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYE-----END PGP SIGNATURE-----\n"
                            .to_string()
                    )
                );
                assert_eq!(addresses[0].BitcoinAddressIndex, Some(13));
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_update_bitcoin_address() {
        let bitcoin_address = ApiBitcoinAddressCreationPayload {
            BitcoinAddress: "bc1q3msh39t8eycqfpyx85yk3rehluhfjly0elp6q4".to_string(),
            BitcoinAddressSignature:
                "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\n6666-----END PGP SIGNATURE-----\n".to_string(),
            BitcoinAddressIndex: 16,
        };

        let mock_server = MockServer::start().await;
        let wallet_id = "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==";
        let wallet_account_id =
            "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==";
        let wallet_account_bitcoin_address_id =
            "8gdVKE1364EL3g0VvQOZKIlh97RoiDS3CfJqiEyaT4T2V1sWtUV8JmUgm0foaHfvCEjVOuE5MqKOM32mp2QEKg==";
        let req_path: String = format!(
            "{}/wallets/{}/accounts/{}/addresses/bitcoin/{}",
            BASE_WALLET_API_V1, wallet_id, wallet_account_id, wallet_account_bitcoin_address_id
        );
        let response_body = serde_json::json!(
        {
            "Code": 1000,
            "WalletBitcoinAddress":
            {
                "ID": "8gdVKE1364EL3g0VvQOZKIlh97RoiDS3CfJqiEyaT4T2V1sWtUV8JmUgm0foaHfvCEjVOuE5MqKOM32mp2QEKg==",
                "WalletID": "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg==",
                "WalletAccountID": "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A==",
                "Fetched": 0,
                "Used": 0,
                "BitcoinAddress": "bc1q3msh39t8eycqfpyx85yk3rehluhfjly0elp6q4",
                "BitcoinAddressSignature": "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\n6666-----END PGP SIGNATURE-----\n",
                "BitcoinAddressIndex": 16
            },
        });
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("PUT"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "BitcoinAddress": "bc1q3msh39t8eycqfpyx85yk3rehluhfjly0elp6q4",
                "BitcoinAddressSignature": "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\n6666-----END PGP SIGNATURE-----\n",
                "BitcoinAddressIndex": 16,
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = BitcoinAddressClient::new(Arc::new(api_client));
        let result = client
            .update_bitcoin_address(
                wallet_id.to_string(),
                wallet_account_id.to_string(),
                wallet_account_bitcoin_address_id.to_string(),
                bitcoin_address,
            )
            .await;
        match result {
            Ok(address) => {
                assert_eq!(
                    address.ID,
                    "8gdVKE1364EL3g0VvQOZKIlh97RoiDS3CfJqiEyaT4T2V1sWtUV8JmUgm0foaHfvCEjVOuE5MqKOM32mp2QEKg=="
                );
                assert_eq!(
                    address.WalletID,
                    "_zuc9hOPmSeNUPoBlvFs2JvjWw_hX4ktpVnqKmpAhh3PcAGXNVJqU_jD2ZoZ_qTteGsa30m8mHG8GiWt_7L0xg=="
                );
                assert_eq!(
                    address.WalletAccountID,
                    "yYzIuZJobta-FCUwbhCdUwCXtn-BLoW0yZvVNJK5MCh0KT-igpGYa3zd_uNz43gKTD9BXrRaDlT4uRhdo70y_A=="
                );
                assert_eq!(address.Fetched, 0);
                assert_eq!(address.Used, 0);
                assert_eq!(
                    address.BitcoinAddress,
                    Some("bc1q3msh39t8eycqfpyx85yk3rehluhfjly0elp6q4".to_string())
                );
                assert_eq!(
                    address.BitcoinAddressSignature,
                    Some(
                        "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\n6666-----END PGP SIGNATURE-----\n"
                            .to_string()
                    )
                );
                assert_eq!(address.BitcoinAddressIndex, Some(16));
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
