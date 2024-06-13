use std::sync::Arc;

use serde::Deserialize;

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_CONTACTS_API_V4,
};

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetContactsResponseBody {
    pub Code: u16,
    pub ContactEmails: Vec<ApiContactEmails>,
    pub Total: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiContactEmails {
    pub ID: String,
    pub Name: String,
    pub Email: String,
    pub CanonicalEmail: String,
    pub IsProton: u32,
}

#[derive(Clone)]
pub struct ContactsClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for ContactsClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_CONTACTS_API_V4
    }
}

impl ContactsClient {
    pub async fn get_contacts(
        &self,
        page_size: Option<u64>,
        page: Option<u64>,
    ) -> Result<Vec<ApiContactEmails>, Error> {
        let mut request = self.get("contacts/emails");

        if let Some(page_size) = page_size {
            request = request.query(("PageSize", page_size.to_string()));
        }

        if let Some(page) = page {
            request = request.query(("Page", page.to_string()));
        }

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetContactsResponseBody>()?;

        Ok(parsed.ContactEmails)
    }
}

#[cfg(test)]
mod tests {

    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::ContactsClient;
    use crate::{
        core::ApiClient,
        tests::utils::{common_api_client, setup_test_connection},
        BASE_CONTACTS_API_V4,
    };

    //TODO:: real api calls need to move to integration tests. with quark commands
    #[tokio::test]
    #[ignore]
    async fn should_get_contacts() {
        let api_client = common_api_client().await;
        let client = ContactsClient::new(api_client);
        let contacts = client.get_contacts(Some(100), Some(0)).await;
        println!("request done: {:?}", contacts);
        assert!(contacts.is_ok());
    }

    #[tokio::test]
    #[ignore]
    #[cfg(feature = "allow-dangerous-env")]
    async fn test_get_contacts_code_1000() {
        use std::env;

        env::set_var("RUST_LOG", "debug");
        tracing_subscriber::fmt::init();

        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "ContactEmails": [
                  {
                    "ID": "p_T6zor7HDtNUkLkXzPLRYAwOs4Ra4phOT84BRpifqNklrtRroqrR1gHhdkDvDsfwmHNKOdFaxiB2lax47CvxQ==",
                    "Name": "Test",
                    "Email": "Test@testmail.com",
                    "IsProton": 0,
                    "CanonicalEmail": "test@testmail.com"
                  },
                  {
                    "ID": "rQMb1pOGxRwLY6iNVgvEMP5kMJW18x12uYKGv-MdqLupYjmcWoawr9mgE62dN7dwBV55Dy9Wl8gCZ0J-1ZB-Tg==",
                    "Name": "free@proton.black",
                    "Email": "free@proton.black",
                    "IsProton": 0,
                    "CanonicalEmail": "free@proton.black"
                  }
                ],
                "Total": 2
              }
        );
        let req_path = format!("{}/contacts/emails", BASE_CONTACTS_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let client = ContactsClient::new(api_client);
        let contacts = client.get_contacts(Some(100), Some(0)).await;
        match contacts {
            Ok(ref vec) => assert!(vec.len() == 2, "Expected the vector to contain 2 contacts."),
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_contacts_deserialize_error() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({});
        let req_path = format!("{}/contacts/emails", BASE_CONTACTS_API_V4);
        let response = ResponseTemplate::new(400).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let session = setup_test_connection(mock_server.uri());
        let client = ContactsClient::new(session);
        let contacts = client.get_contacts(Some(100), Some(0)).await;
        assert!(contacts.is_err());
    }
}
