use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{store::SimpleAuthStore, transports::ReqwestTransportFactory, AppSpec, Product, Session};

pub async fn common_session() -> Arc<RwLock<Session>> {
    let app = AppSpec::new(Product::Unspecified, "web-wallet@5.0.999.999-dev".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());
    let auth = SimpleAuthStore::new("atlas");

    let transport = ReqwestTransportFactory::new();
    let mut session = Session::new_with_transport(auth, app, transport).unwrap();

    session.authenticate("pro", "pro").await.unwrap();
    Arc::new(RwLock::new(session))
}
