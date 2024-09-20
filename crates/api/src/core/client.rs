use std::sync::Arc;

use muon::ProtonRequest;

use super::ToProtonRequest;
use crate::ProtonWalletApiClient;

pub trait ApiClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self;
    fn api_client(&self) -> &Arc<ProtonWalletApiClient>;
    // api base path with api version,
    fn base_url(&self) -> &str;

    fn get(&self, endpoint: impl ToString) -> ProtonRequest {
        return self.build_request(self.base_url(), endpoint).to_get_request();
    }
    fn post(&self, endpoint: impl ToString) -> ProtonRequest {
        return self.build_request(self.base_url(), endpoint).to_post_request();
    }
    fn put(&self, endpoint: impl ToString) -> ProtonRequest {
        return self.build_request(self.base_url(), endpoint).to_put_request();
    }
    fn delete(&self, endpoint: impl ToString) -> ProtonRequest {
        return self.build_request(self.base_url(), endpoint).to_delete_request();
    }
    fn build_request(&self, version: &str, endpoint: impl ToString) -> String {
        return self.api_client().build_full_url(version, endpoint);
    }
}
