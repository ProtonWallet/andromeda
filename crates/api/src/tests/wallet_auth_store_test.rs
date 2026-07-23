use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use muon::client::{Auth, Tokens};
use muon::store::Store;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

use crate::{
    core::ApiClient, tests::utils::test_spec, unleash::UnleashClient, ApiConfig, ProtonWalletApiClient, WalletAuthStore,
};

fn test_auth() -> Auth {
    Auth::internal(
        "test_user_id",
        "test_session_uid",
        Tokens::access("test_access_token", "test_refresh_token", ["wallet", "account"]),
    )
}

fn restored_session_store(server_uri: &str) -> WalletAuthStore {
    WalletAuthStore::from_env_str(server_uri.to_string(), Arc::new(Mutex::new(test_auth())))
}

#[tokio::test]
async fn store_get_auth_returns_none_initially() {
    let store = WalletAuthStore::prod();

    let auth = store.get_auth(&()).await.unwrap();

    assert!(matches!(auth, Auth::None));
}

#[tokio::test]
async fn store_set_auth_persists_internal_session() {
    let mut store = WalletAuthStore::prod();
    let expected = test_auth();

    store.set_auth(&(), expected.clone()).await.unwrap();

    let auth = store.get_auth(&()).await.unwrap();
    assert_eq!(auth.uid(), expected.uid());
    assert_eq!(auth.acc_tok(), expected.acc_tok());
    assert_eq!(auth.ref_tok(), expected.ref_tok());
    assert_eq!(auth.scopes(), expected.scopes());
}

#[tokio::test]
async fn store_set_auth_rejects_none() {
    let mut store = WalletAuthStore::prod();

    let err = store.set_auth(&(), Auth::None).await.unwrap_err();

    assert!(matches!(err, muon::store::StoreError));
    assert!(matches!(store.get_auth(&()).await.unwrap(), Auth::None));
}

#[tokio::test]
async fn store_remove_auth_clears_session_and_returns_previous() {
    let mut store = WalletAuthStore::prod();
    let expected = test_auth();
    store.set_auth(&(), expected.clone()).await.unwrap();

    let removed = store.remove_auth(&()).await.unwrap();

    assert_eq!(removed.as_ref().map(Auth::uid), Some(expected.uid()));
    assert!(matches!(store.get_auth(&()).await.unwrap(), Auth::None));
}

#[tokio::test]
async fn store_remove_auth_on_empty_returns_none() {
    let mut store = WalletAuthStore::prod();

    let removed = store.remove_auth(&()).await.unwrap();

    assert!(removed.is_none());
}

#[tokio::test]
async fn store_get_all_auth_reflects_session_state() {
    let mut store = WalletAuthStore::prod();
    let expected = test_auth();

    assert!(store.get_all_auth().await.unwrap().is_empty());

    store.set_auth(&(), expected.clone()).await.unwrap();

    let all = store.get_all_auth().await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all.get(&()).map(Auth::uid), Some(expected.uid()));
}

#[tokio::test]
async fn store_set_all_auth_with_empty_map_clears_session() {
    let mut store = WalletAuthStore::prod();
    store.set_auth(&(), test_auth()).await.unwrap();

    store.set_all_auth(HashMap::new()).await.unwrap();

    assert!(matches!(store.get_auth(&()).await.unwrap(), Auth::None));
}

#[tokio::test]
async fn store_remove_all_auth_clears_session() {
    let mut store = WalletAuthStore::prod();
    store.set_auth(&(), test_auth()).await.unwrap();

    store.remove_all_auth().await.unwrap();

    assert!(matches!(store.get_auth(&()).await.unwrap(), Auth::None));
}

#[tokio::test]
async fn from_config_uses_restored_session_without_unauth_call() {
    let mock_server = MockServer::start().await;
    let store = Box::new(restored_session_store(&mock_server.uri()));

    Mock::given(method("GET"))
        .and(path("/feature/v2/frontend"))
        .and(header("x-pm-uid", "test_session_uid"))
        .and(header("authorization", "Bearer test_access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = ApiConfig {
        spec: test_spec(),
        url_prefix: None,
        env: Some(mock_server.uri()),
        store: Some(store),
        auth: None,
        proxy: None,
    };

    let api_client = ProtonWalletApiClient::from_config(config).unwrap();
    let client = UnleashClient::new(Arc::new(api_client));

    let response = client.fetch_toggles().await.unwrap();

    assert_eq!(response.status_code, 200);
    mock_server.verify().await;
}

#[tokio::test]
async fn from_config_refreshes_session_after_unauthorized() {
    let mock_server = MockServer::start().await;
    let store = Box::new(restored_session_store(&mock_server.uri()));

    Mock::given(method("GET"))
        .and(path("/feature/v2/frontend"))
        .and(header("x-pm-uid", "test_session_uid"))
        .and(header("authorization", "Bearer test_access_token"))
        .respond_with(ResponseTemplate::new(401))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/auth/v4/refresh"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Code": 1000,
            "UID": "test_session_uid",
            "AccessToken": "refreshed_access_token",
            "RefreshToken": "refreshed_refresh_token",
            "TokenType": "Bearer",
            "Scopes": ["wallet", "account"]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/feature/v2/frontend"))
        .and(header("x-pm-uid", "test_session_uid"))
        .and(header("authorization", "Bearer refreshed_access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = ApiConfig {
        spec: test_spec(),
        url_prefix: None,
        env: Some(mock_server.uri()),
        store: Some(store),
        auth: None,
        proxy: None,
    };

    let api_client = ProtonWalletApiClient::from_config(config).unwrap();
    let client = UnleashClient::new(Arc::new(api_client));

    let response = client.fetch_toggles().await.unwrap();

    assert_eq!(response.status_code, 200);
    mock_server.verify().await;
}
