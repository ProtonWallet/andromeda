use std::sync::Arc;

use crate::{ApiConfig, ProtonWalletApiClient};

pub fn test_spec() -> (String, String) {
    ("web-wallet@5.0.999.999-dev".to_string(),"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string())
}

pub fn setup_test_connection(url: String) -> Arc<ProtonWalletApiClient> {
    let config = ApiConfig {
        spec: Some(test_spec()),
        url_prefix: None,
        env: Some(url),
        store: None,
        auth: None,
    };
    Arc::new(ProtonWalletApiClient::from_config(config).unwrap())
}

pub async fn common_api_client() -> Arc<ProtonWalletApiClient> {
    let config = ApiConfig {
        spec: Some(test_spec()),
        url_prefix: None,
        env: None,
        store: None,
        auth: None,
    };
    let api = ProtonWalletApiClient::from_config(config).unwrap();
    api.login("pro", "pro").await.unwrap();
    Arc::new(api)
}
