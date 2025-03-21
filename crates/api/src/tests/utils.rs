use std::sync::Arc;

use crate::{ApiConfig, ProtonWalletApiClient};

pub fn test_spec() -> (String, String) {
    ("web-wallet@5.0.999.999-dev".to_string(),"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string())
}

pub fn setup_test_connection(url: String) -> Arc<ProtonWalletApiClient> {
    let config = ApiConfig {
        spec: test_spec(),
        url_prefix: None,
        env: Some(url),
        store: None,
        auth: None,
        proxy: None,
    };

    Arc::new(ProtonWalletApiClient::from_config(config).unwrap())
}

pub async fn common_api_client() -> Arc<ProtonWalletApiClient> {
    let config = ApiConfig {
        spec: test_spec(),
        url_prefix: None,
        env: None,
        store: None,
        auth: None,
        proxy: None,
    };
    let api = ProtonWalletApiClient::from_config(config).unwrap();
    api.login("bart", "bart").await.unwrap();
    Arc::new(api)
}

#[doc(hidden)]
#[macro_export]
macro_rules! read_mock_file {
    ($filename:expr) => {{
        use std::{fs::File, io::Read};

        let mut file = File::open(format!("./src/tests/mocks/{}.json", $filename)).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        contents
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! read_mock_raw_file {
    ($filename:expr) => {{
        use std::{fs::File, io::Read};

        let mut file = File::open(format!("./src/tests/mocks/{}.raw", $filename)).unwrap();
        let mut contents = Vec::new();

        file.read_to_end(&mut contents).unwrap();
        contents
    }};
}
