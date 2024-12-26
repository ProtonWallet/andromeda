use std::sync::Arc;

use muon::ProtonRequest;

use super::ToProtonRequest;
use crate::{get_default_time_constraint, ProtonWalletApiClient, DEFAULT_INTERACTIVITY, DEFAULT_SERVICE_TYPE};

pub trait ApiClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self;
    fn api_client(&self) -> &Arc<ProtonWalletApiClient>;
    // api base path with api version,
    fn base_url(&self) -> &str;

    fn get(&self, endpoint: impl ToString) -> ProtonRequest {
        return self
            .build_request(self.base_url(), endpoint)
            .to_get_request()
            .allowed_time(get_default_time_constraint())
            .service_type(DEFAULT_SERVICE_TYPE, true);
    }
    fn post(&self, endpoint: impl ToString) -> ProtonRequest {
        return self
            .build_request(self.base_url(), endpoint)
            .to_post_request()
            .allowed_time(get_default_time_constraint())
            .service_type(DEFAULT_INTERACTIVITY, true);
    }
    fn put(&self, endpoint: impl ToString) -> ProtonRequest {
        return self
            .build_request(self.base_url(), endpoint)
            .to_put_request()
            .allowed_time(get_default_time_constraint())
            .service_type(DEFAULT_INTERACTIVITY, true);
    }
    fn delete(&self, endpoint: impl ToString) -> ProtonRequest {
        return self
            .build_request(self.base_url(), endpoint)
            .to_delete_request()
            .allowed_time(get_default_time_constraint())
            .service_type(DEFAULT_INTERACTIVITY, true);
    }
    fn build_request(&self, version: &str, endpoint: impl ToString) -> String {
        return self.api_client().build_full_url(version, endpoint);
    }
}
