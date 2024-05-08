use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Session};
use serde::Deserialize;

use crate::{error::Error, proton_response_ext::ProtonResponseExt, BASE_CONTACTS_API_V4};

#[derive(Clone)]
pub struct ContactsClient {
    session: Arc<RwLock<Session>>,
}

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

impl ContactsClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_contacts(&self, page_size: u64, page: u64) -> Result<Vec<ApiContactEmails>, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!(
                "{}/contacts/emails?PageSize={}&Page={}",
                BASE_CONTACTS_API_V4, page_size, page,
            ),
        );

        let response = self.session.read().await.bind(request)?.send().await?;
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
    use crate::{utils::common_session, utils_test::setup_test_connection, BASE_CONTACTS_API_V4};

    //TODO:: real api calls need to move to integration tests. with quark commands
    #[tokio::test]
    #[ignore]
    async fn should_get_contacts() {
        let session = common_session().await;
        let client = ContactsClient::new(session);
        let contacts = client.get_contacts(100, 0).await;
        println!("request done: {:?}", contacts);
    }

    #[tokio::test]
    async fn test_get_contacts_code_1000() {
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
        let session = setup_test_connection(mock_server.uri());
        let client = ContactsClient::new(session);
        let contacts = client.get_contacts(100, 0).await;
        match contacts {
            Ok(ref vec) => assert!(vec.len() == 2, "Expected the vector to contain 2 contacts."),
            Err(_) => panic!("Expected Ok variant but got Err."),
        }
    }

    #[tokio::test]
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
        let contacts = client.get_contacts(100, 0).await;
        assert!(contacts.is_err());
    }
}
