use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Response, Session};
use serde::Deserialize;
use std::sync::Arc;

use crate::{error::Error, BASE_CONTACTS_API_V4};

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
                "{}/emails?PageSize={}&Page={}",
                BASE_CONTACTS_API_V4,
                page_size.to_string(),
                page.to_string(),
            ),
        );

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetContactsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.ContactEmails)
    }
}

#[cfg(test)]
mod tests {
    use super::ContactsClient;
    use crate::utils::common_session;

    #[tokio::test]
    #[ignore]
    async fn should_get_contacts() {
        let session = common_session().await;
        let client = ContactsClient::new(session);

        let contacts = client.get_contacts(100, 0).await;

        println!("request done: {:?}", contacts);
    }
}
