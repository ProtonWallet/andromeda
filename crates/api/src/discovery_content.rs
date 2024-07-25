use std::sync::Arc;

use serde::Deserialize;

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

#[derive(Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct Content {
    #[serde(alias = "title")]
    pub Title: String,
    #[serde(alias = "link")]
    pub Link: String,
    #[serde(alias = "description")]
    pub Description: String,
    #[serde(alias = "pubDate")]
    pub PubDate: i64,
    #[serde(alias = "author")]
    pub Author: String,
    #[serde(alias = "category")]
    pub Category: String,
}

#[derive(Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct GetDiscoveryContentResponseBody {
    pub Code: i32,
    pub DiscoverContent: Vec<Content>,
}

#[derive(Clone)]
pub struct DiscoverContentClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for DiscoverContentClient {
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

impl DiscoverContentClient {
    pub async fn get_discovery_contents(&self) -> Result<Vec<Content>, Error> {
        let request = self.get("discover-content");
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetDiscoveryContentResponseBody>()?;
        Ok(parsed.DiscoverContent)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        core::ApiClient,
        discovery_content::{Content, DiscoverContentClient},
        tests::utils::setup_test_connection_arc,
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    async fn test_get_discovery_content() {
        const DOMAIN: &str = "https://proton.me";
        const SUBDOMAIN_WALLET: &str = "wallet";
        const SUBDOMAIN_BLOG: &str = "blog";
        let mock_server = MockServer::start().await;
        let json_body = serde_json::json!(
        {
            "Code": 1000,
            "DiscoverContent": [
                {
                    "Title": "Bitcoin guide for newcomers",
                    "Link": format!("https://{}/{}/bitcoin-guide-for-newcomers", DOMAIN, SUBDOMAIN_WALLET),
                    "Description": "We review some important history and features of Bitcoin for newcomers. We also look at how Bitcoin enables financial sovereignty and freedom. Finally, we explore the challenges facing Bitcoin and its future potential.",
                    "PubDate": 1721701601,
                    "Author": "Proton Team",
                    "Category": "Bitcoin basics"
                  },
                  {
                    "title": "What is Bitcoin?",
                    "Link": format!("https://{}/{}/what-is-bitcoin", DOMAIN, SUBDOMAIN_BLOG),
                    "description": "Bitcoin is an innovative payment network that leverages peer-to-peer transactions to remove the need for a central bank. Bitcoin has revolutionized the core principles of value exchange by showing that a network of fully independent nodes can operate payments in a trustless and secure way.",
                    "pubDate": 1721701601,
                    "author": "Proton Team",
                    "category": "Bitcoin basics"
                  },
            ]
        });

        let req_path: String = format!("{}/discover-content", BASE_WALLET_API_V1);
        let response = ResponseTemplate::new(200).set_body_json(json_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let api_client = setup_test_connection_arc(mock_server.uri());
        let client = DiscoverContentClient::new(api_client);
        let data = client.get_discovery_contents().await;

        println!("discover content data:{:?}", data);
        assert_eq!(
            data.unwrap(),
            vec![
                Content {
                    Title: "Bitcoin guide for newcomers".to_owned(),
                    Link: format!("https://{}/{}/bitcoin-guide-for-newcomers", DOMAIN, SUBDOMAIN_WALLET).to_owned(),
                    Description: "We review some important history and features of Bitcoin for newcomers. We also look at how Bitcoin enables financial sovereignty and freedom. Finally, we explore the challenges facing Bitcoin and its future potential.".to_owned(),
                    PubDate: 1721701601,
                    Author: "Proton Team".to_owned(),
                    Category: "Bitcoin basics".to_owned(),
                },
                Content {
                    Title: "What is Bitcoin?".to_owned(),
                    Link: format!("https://{}/{}/what-is-bitcoin", DOMAIN, SUBDOMAIN_BLOG).to_owned(),
                    Description: "Bitcoin is an innovative payment network that leverages peer-to-peer transactions to remove the need for a central bank. Bitcoin has revolutionized the core principles of value exchange by showing that a network of fully independent nodes can operate payments in a trustless and secure way.".to_owned(),
                    PubDate: 1721701601,
                    Author: "Proton Team".to_owned(),
                    Category: "Bitcoin basics".to_owned(),
                },
            ]
        )
    }
}
