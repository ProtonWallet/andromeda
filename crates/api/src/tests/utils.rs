use std::sync::{Arc, Mutex};

use muon::{App, Auth, Client};

use crate::{ProtonWalletApiClient, WalletAuthStore};

pub fn setup_test_connection(url: String) -> Arc<ProtonWalletApiClient> {
    let app = App::new("web-wallet@5.0.999.999-dev").unwrap().with_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
    let auth = WalletAuthStore::from_env_str(url, Arc::new(Mutex::new(Auth::None)));
    let session = Client::new(app, auth).unwrap();
    Arc::new(ProtonWalletApiClient::from_session(session, None))
}

pub async fn common_api_client() -> Arc<ProtonWalletApiClient> {
    let session = {
        let auth = WalletAuthStore::atlas(None);
        let app = App::new("web-wallet@5.0.999").unwrap().with_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
        Client::new(app, auth).unwrap()
    };
    let api = ProtonWalletApiClient::from_session(session, None);
    api.login("pro", "pro").await.unwrap();
    Arc::new(api)
}
