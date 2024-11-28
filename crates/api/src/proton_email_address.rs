use std::sync::Arc;

use serde::Deserialize;

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_CORE_API_V4,
};

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetApiAllKeyResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Address: ApiAllKeyAddress,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiAllKeyAddress {
    pub Keys: Vec<ApiAllKeyAddressKey>,
    // skipping `SignedKeyList` from API document, will add it once we need to use it
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiAllKeyAddressKey {
    pub Flags: u32,
    pub PublicKey: String,
    pub Source: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonAddress {
    pub ID: String,
    pub DomainID: Option<String>,
    pub Email: String,
    pub Status: u32,
    pub Type: u32,
    pub Receive: u32,
    pub Send: u32,
    pub DisplayName: String,
    pub Keys: Option<Vec<ApiProtonAddressKey>>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonAddressKey {
    pub ID: String,
    pub Version: u32,
    pub PublicKey: String,
    pub PrivateKey: Option<String>,
    pub Token: Option<String>,
    pub Signature: Option<String>,
    pub Primary: u32,
    pub Active: u32,
    pub Flags: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetApiProtonAddressesResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Addresses: Vec<ApiProtonAddress>,
}

#[derive(Clone)]
pub struct ProtonEmailAddressClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for ProtonEmailAddressClient {
    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_CORE_API_V4
    }

    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }
}

impl ProtonEmailAddressClient {
    pub async fn get_proton_email_addresses(&self) -> Result<Vec<ApiProtonAddress>, Error> {
        let request = self.get("addresses");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetApiProtonAddressesResponseBody>()?;

        Ok(parsed.Addresses)
    }

    pub async fn get_all_public_keys(
        &self,
        email: String,
        internal_only: Option<u8>,
    ) -> Result<Vec<ApiAllKeyAddressKey>, Error> {
        let mut request = self.get("keys/all").query(("Email", email));
        if let Some(intenal) = internal_only {
            request = request.query(("InternalOnly", intenal.to_string()));
        }

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetApiAllKeyResponseBody>()?;

        Ok(parsed.Address.Keys)
    }
}

#[cfg(test)]
mod tests {

    use super::ProtonEmailAddressClient;
    use crate::{
        core::ApiClient, read_mock_file, tests::utils::common_api_client, tests::utils::setup_test_connection,
        BASE_CORE_API_V4,
    };
    use std::sync::Arc;
    use wiremock::{
        matchers::{body_json, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_proton_email_addresses() {
        let api_client = common_api_client().await;
        let client = ProtonEmailAddressClient::new(api_client);

        let proton_email_addresses = client.get_proton_email_addresses().await;

        println!("request done: {:?}", proton_email_addresses);
        assert!(proton_email_addresses.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_all_public_keys() {
        let api_client = common_api_client().await;
        let client = ProtonEmailAddressClient::new(api_client);

        let all_public_keys = client
            .get_all_public_keys(String::from("pro@proton.black"), Some(1))
            .await;

        println!("request done: {:?}", all_public_keys);
        assert!(all_public_keys.is_ok());
    }

    #[tokio::test]
    async fn test_get_proton_email_addresses_success() {
        let mock_server = MockServer::start().await;
        let contents = read_mock_file!("get_proton_email_addresses_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        let req_path: String = format!("{}/addresses", BASE_CORE_API_V4);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = ProtonEmailAddressClient::new(Arc::new(api_client));
        let result = client.get_proton_email_addresses().await;
        match result {
            Ok(proton_addresses) => {
                assert_eq!(proton_addresses.len(), 3);
                // first proton address
                assert_eq!(
                    proton_addresses[0].ID,
                    "E0QUig1OKNXKfL4-tC78xFNHD8kAw0oBj0HUsxWcHAdtCh8XxojpvX6ApkC4VlgCn5miWpqwH9K0Trj4yF2aLg=="
                );
                assert_eq!(
                    proton_addresses[0].DomainID,
                    Some(
                        "-Bpgivr5H2qGDRiUQ4-7gm5YLf215MEgZCdzOtLW5psxgB8oNc8OnoFRykab4Z23EGEW1ka3GtQPF9xwx9-VUA=="
                            .to_string()
                    )
                );
                assert_eq!(proton_addresses[0].Email, "test.1@proton.me");
                assert_eq!(proton_addresses[0].Status, 1);
                assert_eq!(proton_addresses[0].Type, 1);
                assert_eq!(proton_addresses[0].Receive, 1);
                assert_eq!(proton_addresses[0].Send, 1);
                assert_eq!(proton_addresses[0].DisplayName, "");
                assert!(proton_addresses[0].Keys.is_some());
                assert_eq!(proton_addresses[0].Keys.as_ref().unwrap().len(), 1);

                // second proton address
                assert_eq!(
                    proton_addresses[1].ID,
                    "GeGNYlVUWbRxtso10QmSXbjkY5QgPHjAZnOcpgzxNf038z8eEnQcmyNcB_8LexS-HnPe3e6ehvpkbUxeoylrhg=="
                );
                assert_eq!(
                    proton_addresses[1].DomainID,
                    Some(
                        "-Bpgivr5H2qGDRiUQ4-7gm5YLf215MEgZCdzOtLW5psxgB8oNc8OnoFRykab4Z23EGEW1ka3GtQPF9xwx9-VUA=="
                            .to_string()
                    )
                );
                assert_eq!(proton_addresses[1].Email, "test.2@proton.me");
                assert_eq!(proton_addresses[1].Status, 1);
                assert_eq!(proton_addresses[1].Type, 2);
                assert_eq!(proton_addresses[1].Receive, 1);
                assert_eq!(proton_addresses[1].Send, 1);
                assert_eq!(proton_addresses[1].DisplayName, "test.2");
                assert!(proton_addresses[1].Keys.is_some());
                assert_eq!(proton_addresses[1].Keys.as_ref().unwrap().len(), 1);

                // third proton address
                assert_eq!(
                    proton_addresses[2].ID,
                    "EnpadS8CXQYnChPQNhuOHZBdaVPtkXUJY6o25O1MhQxMPxuChxMdfQffGuMqs2R8hhcVKVxQgEA5reQ_obb0AA=="
                );
                assert_eq!(
                    proton_addresses[2].DomainID,
                    Some(
                        "-Bpgivr5H2qGDRiUQ4-7gm5YLf215MEgZCdzOtLW5psxgB8oNc8OnoFRykab4Z23EGEW1ka3GtQPF9xwx9-VUA=="
                            .to_string()
                    )
                );
                assert_eq!(proton_addresses[2].Email, "test.3@proton.me");
                assert_eq!(proton_addresses[2].Status, 1);
                assert_eq!(proton_addresses[2].Type, 2);
                assert_eq!(proton_addresses[2].Receive, 1);
                assert_eq!(proton_addresses[2].Send, 1);
                assert_eq!(proton_addresses[2].DisplayName, "test.3");
                assert!(proton_addresses[2].Keys.is_some());
                assert_eq!(proton_addresses[2].Keys.as_ref().unwrap().len(), 1);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_get_all_public_keys_success() {
        let mock_server = MockServer::start().await;
        let email = "test@proton.me";
        let req_path: String = format!("{}/keys/all", BASE_CORE_API_V4);
        let contents = read_mock_file!("get_all_public_keys_1000_body");
        let response = ResponseTemplate::new(200).set_body_string(contents);
        Mock::given(method("GET"))
            .and(path(req_path))
            .and(query_param("Email", email))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = ProtonEmailAddressClient::new(Arc::new(api_client));
        let result = client.get_all_public_keys(email.to_string(), None).await;
        match result {
            Ok(keys) => {
                assert_eq!(keys.len(), 1);
                assert_eq!(keys[0].Flags, 3);
                assert_eq!(keys[0].PublicKey, "-----BEGIN PGP PUBLIC KEY BLOCK-----\nVersion: ProtonMail\n\nxjMEZqIR1hYJKwYBBAHaRw8BAQdAg53JRcPL/EkpeYYKX5qAndW0T8IpKh25\nJDjDAoyONVfNKXdpbGwua2FiaUBwcm90b24ubWUgPHdpbGwua2FiaUBwcm90\nb24ubWU+wo8EExYIAEEFAmaiEdYJEJSe7m6ROP3SFiEEAQh6xxLmaiflg8yV\nlJ7ubpE4/dICGwMCHgECGQEDCwkHAhUIAxYAAgUnCQIHAgAA2u8A+QEo4shz\nGkUB0iEj+Nq/fzpm9GBTVfZZ0MxrwU2HJIuBAQCEHHoXA2ggmF0OmofJAvt7\nHpwrMQY4xZSCdr/YAc7xCMKoBBAWCABaBQJmo2NYCRDYBsGvWXjoxxYhBAqG\nUv5dUzhgV4mf6dgGwa9ZeOjHLBxvcGVucGdwLWNhQHByb3Rvbi5tZSA8b3Bl\nbnBncC1jYUBwcm90b24ubWU+BYMA7U4AAADf8AEAiQY2GNiwbBArLKBKStGq\npjJj1awQ+2Dz/ShBmd/WAc4BALsgfE8BKfgF/XEpUJosxi0AYrCX+FjWlFmZ\nXJNZkwoAzjgEZqIR1hIKKwYBBAGXVQEFAQEHQI0AcNDf/dmvYkuICaB18YHa\nGi+Ngqu/fguzHTUeQ/YHAwEKCcJ4BBgWCAAqBQJmohHWCRCUnu5ukTj90hYh\nBAEIescS5mon5YPMlZSe7m6ROP3SAhsMAABUaQEA6eK3qyBs0p9J0rmp2B9p\nKkFr90LVND5OJPLE+qe4P5MBAM0W7NFbrgTR6osJWX1uDfQh4zP86PWhKJr2\nVptEyPsA\n=W4qM\n-----END PGP PUBLIC KEY BLOCK-----\n");
                assert_eq!(keys[0].Source, 0);
                return;
            }
            Err(e) => panic!("Got Err. {:?}", e),
        }
    }
}
