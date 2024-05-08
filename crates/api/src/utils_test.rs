use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{
    environment::ApiEnv, store::SimpleAuthStore, transports::ReqwestTransportFactory, AppSpec, Product, Session,
};

#[derive(Debug, Default)]
pub struct TestEnv {
    name: String,
    base: String,
}

impl TestEnv {
    const PINS: &'static [&'static str] = &[];

    #[must_use]
    pub fn new(url: String) -> Self {
        Self {
            name: "local:test".to_string(),
            base: url,
        }
    }
}

impl ApiEnv for TestEnv {
    fn name(&self) -> &str {
        &self.name
    }

    fn base(&self, _: &Product) -> &str {
        &self.base
    }

    fn pins(&self) -> &[&'static str] {
        Self::PINS
    }
}

pub fn setup_test_connection(url: String) -> Arc<RwLock<Session>> {
    let app = AppSpec::new(Product::Wallet, "web-wallet@5.0.999.999-dev".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());
    let auth = SimpleAuthStore::new("atlas");
    let transport = ReqwestTransportFactory::new();
    let env = TestEnv::new(url);
    let session = Session::new_dangerous(auth, app, transport, env).unwrap();
    Arc::new(RwLock::new(session))
}
