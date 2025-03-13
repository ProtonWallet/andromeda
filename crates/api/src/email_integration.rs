use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Debug, Deserialize, Clone)]
#[allow(non_snake_case)]
pub struct ApiWalletBitcoinAddressLookup {
    pub BitcoinAddress: Option<String>,
    pub BitcoinAddressSignature: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateBitcoinAddressRequestBody {
    pub Email: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct LookupBitcoinAddressResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddress: ApiWalletBitcoinAddressLookup,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct CreateBitcoinAddressRequestResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
}

#[derive(Clone)]
pub struct EmailIntegrationClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for EmailIntegrationClient {
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

impl EmailIntegrationClient {
    pub async fn lookup_bitcoin_address(&self, email: String) -> Result<ApiWalletBitcoinAddressLookup, Error> {
        let request = self.get("emails/lookup").query(("Email", email));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<LookupBitcoinAddressResponseBody>()?;

        Ok(parsed.WalletBitcoinAddress)
    }

    pub async fn create_bitcoin_addresses_request(&self, email: String) -> Result<(), Error> {
        let payload = CreateBitcoinAddressRequestBody { Email: email };

        let request = self.post("emails/requests").body_json(payload)?;

        let response = self.api_client.send(request).await?;
        response.parse_response::<CreateBitcoinAddressRequestResponseBody>()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EmailIntegrationClient;
    use crate::{
        core::ApiClient, tests::utils::common_api_client, tests::utils::setup_test_connection, BASE_WALLET_API_V1,
    };
    use wiremock::{
        matchers::{body_json, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    #[ignore]
    async fn should_lookup_bitcoin_address() {
        let api_client = common_api_client().await;
        let client = EmailIntegrationClient::new(api_client);

        let bitcoin_address = client.lookup_bitcoin_address(String::from("pro@proton.black")).await;

        println!("request done: {:?}", bitcoin_address);
        assert!(bitcoin_address.is_ok());
    }

    #[tokio::test]
    async fn test_lookup_bitcoin_address_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({
            "Code": 1000,
            "WalletBitcoinAddress": {
                "BitcoinAddress": "bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5",
                "BitcoinAddressSignature": "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYEABYIAFgFgmc0QMEJkJSe7m6ROP3SMJSAAAAAABEAFmNvbnRleHRAcHJv\ndG9uLmNod2FsbGV0LmJpdGNvaW4tYWRkcmVzcxYhBAEIescS5mon5YPMlZSe\n7m6ROP3SAABsiQD/fIy6f8QZg0qHT5P7A2e8mtgdujqOjTEibcXolWzhBiUA\n/0icJlt6LONWMOq+QGhANT/F8+dIha4ZLFi2E8J0kmMP\n=HBA3\n-----END PGP SIGNATURE-----\n"
            }
        });
        let email = "test@proton.me";
        let req_path: String = format!("{}/emails/lookup", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("Email", email))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = EmailIntegrationClient::new(api_client);
        let result = client.lookup_bitcoin_address(email.to_string()).await;
        match result {
            Ok(wallet_bitcoin_address) => {
                assert_eq!(
                    wallet_bitcoin_address.BitcoinAddress.unwrap(),
                    "bc1qjxuszfj2xamdmfnqrhljfnyv2cg5zxdgytlnx5"
                );
                assert_eq!(
                    wallet_bitcoin_address.BitcoinAddressSignature.unwrap(),
                    "-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwqYEABYIAFgFgmc0QMEJkJSe7m6ROP3SMJSAAAAAABEAFmNvbnRleHRAcHJv\ndG9uLmNod2FsbGV0LmJpdGNvaW4tYWRkcmVzcxYhBAEIescS5mon5YPMlZSe\n7m6ROP3SAABsiQD/fIy6f8QZg0qHT5P7A2e8mtgdujqOjTEibcXolWzhBiUA\n/0icJlt6LONWMOq+QGhANT/F8+dIha4ZLFi2E8J0kmMP\n=HBA3\n-----END PGP SIGNATURE-----\n"
                );
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_create_bitcoin_addresses_request_success() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({
            "Code": 1000,
        });
        let email = "test@proton.me";
        let req_path: String = format!("{}/emails/requests", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .and(body_json(serde_json::json!(
            {
                "Email": email,
            })))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = EmailIntegrationClient::new(api_client);
        let result = client.create_bitcoin_addresses_request(email.to_string()).await;
        assert!(result.is_ok());
    }
}
