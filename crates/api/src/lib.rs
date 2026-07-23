use core::ApiClient;
use std::{
    env,
    sync::{Arc, Mutex},
    time::Duration,
};

use address::AddressClient;
use bitcoin_address::BitcoinAddressClient;
use block::BlockClient;
use contacts::ContactsClient;
use discovery_content::DiscoverContentClient;
use email_integration::EmailIntegrationClient;
use error::Error;
use event::EventClient;
use exchange_rate::ExchangeRateClient;
use invite::InviteClient;
use log::info;
#[cfg(not(target_arch = "wasm32"))]
use muon::common::{ConstProxy, Endpoint, Scheme};
pub use muon::{
    app::Product,
    client::{flow::LoginFlow, Auth, Tokens},
    common::ServiceType,
    env::EnvId,
    rest::core as CoreAPI,
    store::{DynStore, Store, StoreError},
    util::ProtonRequestExt,
    App, Client, Error as MuonError, ProtonRequest, ProtonResponse, Session, GET,
};
use muon::{
    client::flow::ForkFlowResult,
    common::Host,
    tls::{TlsCert, Verifier, VerifyRes},
};
use network::NetworkClient;
use payment_gateway::PaymentGatewayClient;
use price_graph::PriceGraphClient;
use proton_email_address::ProtonEmailAddressClient;
pub use proton_users::ProtonUsersClient;
use settings::SettingsClient;
use transaction::TransactionClient;
use wallet::WalletClient;
// expose muon's jni. it needs to be matched when use in client
#[cfg(target_os = "android")]
pub use {
    muon::tls::java_init as init_android,
    muon::tls::{
        objects::{JClass, JObject},
        sys::jboolean,
        JNIEnv,
    },
};

pub use crate::{
    core::WalletAuthStore,
    proton_users::{ChildSession, UserData},
};

#[cfg(feature = "quark")]
pub mod proton_quark;

pub mod tests;

pub mod address;
pub mod bitcoin_address;
pub mod block;
pub mod contacts;
pub mod discovery_content;
pub mod email_integration;
pub mod error;
pub mod event;
pub mod exchange_rate;
pub mod invite;
pub mod network;
pub mod payment_gateway;
pub mod price_graph;
pub mod settings;
pub mod transaction;
pub mod wallet;

pub mod proton_email_address;
pub mod proton_settings;
pub mod proton_users;

pub mod core;
pub mod unleash;
pub mod wallet_ext;

pub const BASE_WALLET_API_V1: &str = "wallet/v1";
pub const BASE_CORE_API_V4: &str = "core/v4";
pub const BASE_CORE_API_V5: &str = "core/v5";
pub const BASE_CONTACTS_API_V4: &str = "contacts/v4";

pub const DEFAULT_SERVICE_TYPE: ServiceType = ServiceType::Normal;
pub const DEFAULT_INTERACTIVITY: ServiceType = ServiceType::Interactive;

// get default time constraint from env, or we will use 30 as default time constraint
pub fn get_default_time_constraint() -> Duration {
    let time_constraint = env::var("DEFAULT_TIME_CONSTRAINT")
        .ok()
        .and_then(|val| val.parse::<u64>().ok())
        .unwrap_or(30); // set default time constraint to 30s

    Duration::from_secs(time_constraint)
}

/// An API client providing interfaces to send authenticated http requests to
/// Wallet backend
///
/// ```no_run
/// use andromeda_api::ProtonWalletApiClient;
/// tokio_test::block_on(async {
///     let api_client: ProtonWalletApiClient = ProtonWalletApiClient::default();
///     api_client.login("pro", "pro").await.unwrap();
///     let network = api_client.clients().network.get_network().await.unwrap();
///     println!("network fetch: {:?}", network);
/// });
/// ```
#[derive(Clone)]
pub struct ProtonWalletApiClient {
    session: Session<()>,
    url_prefix: Option<String>,
    // cache the env, when doing the fork, we need to target same env
    env: Option<String>,
}

#[derive(Debug)]
pub struct ApiConfig {
    /// A tupple composed of `app_version` and `user_agent`
    pub spec: (String, String),
    /// The api client initial auth data
    pub auth: Option<Auth>,
    /// An optional prefix to use on every api endpoint call
    pub url_prefix: Option<String>,
    /// The env for the api client
    /// could be [altas, prod, or rul link]
    pub env: Option<String>,
    /// The muon auth store. web doesn't need but flutter side needs
    pub store: Option<DynStore<()>>,
    /// The proxy address. Enable `allow-dangerous-env`` feature to use this
    pub proxy: Option<ProxyConfig>,
}

#[derive(Debug)]
pub struct ProxyConfig {
    pub host: String,
    pub port: u16,
}

pub struct Clients {
    pub block: BlockClient,
    pub network: NetworkClient,
    pub settings: SettingsClient,
    pub transaction: TransactionClient,
    pub wallet: WalletClient,
    pub event: EventClient,
    pub address: AddressClient,
    pub payment_gateway: PaymentGatewayClient,
    pub price_graph: PriceGraphClient,
    pub proton_email_address: ProtonEmailAddressClient,
    pub exchange_rate: ExchangeRateClient,
    pub bitcoin_address: BitcoinAddressClient,
    pub contacts: ContactsClient,
    pub email_integration: EmailIntegrationClient,
    pub invite: InviteClient,
    pub discover_content: DiscoverContentClient,
}

impl ProtonWalletApiClient {
    /// Builds a new api client from a config struct (blocking).
    ///
    /// This is the correct entry point for sync contexts such as the WASM
    /// `#[wasm_bindgen(constructor)]` in `andromeda-wasm`. On native targets it
    /// delegates to [`Self::from_config_async`] via [`block_on`].
    ///
    /// env can be a custom url like localhost or 127.0.0.1, but it requires
    /// `allow-dangerous-env` feature to be enabled
    ///
    /// ```rust
    /// use andromeda_api::{ProtonWalletApiClient, ApiConfig, WalletAuthStore};
    /// use muon::client::{Auth, Tokens};
    /// let auth = Auth::internal("userid", "uid", Tokens::access("acc_tok", "ref_tok", ["scopes"]));
    /// let config = ApiConfig {
    ///     spec: (String::from("android-wallet/1.0.0"), String::from("ProtonWallet/plus-agent-details")),
    ///     auth: Some(auth),
    ///     env: Some("atlas".to_string()),
    ///     url_prefix: None,
    ///     store: None,
    ///     proxy: None,
    /// };
    /// let api_client = ProtonWalletApiClient::from_config(config);
    /// ```
    pub fn from_config(config: ApiConfig) -> Result<Self, Error> {
        block_on(Self::from_config_async(config))
    }

    /// Async variant of [`Self::from_config`].
    ///
    /// Prefer this when already on a tokio runtime (e.g. Flutter FRB async handlers)
    /// to avoid `block_on` deadlocks. WASM browser code should keep using the sync
    /// [`Self::from_config`] from constructors; both paths share this implementation.
    pub async fn from_config_async(config: ApiConfig) -> Result<Self, Error> {
        let env: String = config.env.clone().unwrap_or("atlas".to_string());

        let (app_version, user_agent) = config.spec;
        let app = App::new(app_version)?.with_user_agent(user_agent);

        let store = config.store.unwrap_or_else(|| {
            let auth = config.auth.unwrap_or(Auth::None);
            if config.proxy.is_none() {
                Box::new(WalletAuthStore::from_env_str(env, Arc::new(Mutex::new(auth))))
            } else {
                Box::new(WalletAuthStore::from_custom_env_str(env, Arc::new(Mutex::new(auth))))
            }
        });

        let mut builder = Client::builder(app, store).await;
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(proxy) = config.proxy {
            if let Ok(host) = Host::direct(proxy.host) {
                builder = builder.verifier(UnSafeVerifier);
                builder = builder.proxy(ConstProxy::new(Endpoint::new(Scheme::Http, host, proxy.port)));
            }
        }
        let client = builder.build()?;
        let session = match client.get_session(()).await {
            Some(session) => session,
            None => client.new_session_without_credentials(()).await?,
        };

        Ok(Self {
            session,
            url_prefix: config.url_prefix,
            env: config.env,
        })
    }

    pub fn clients(&self) -> Clients {
        let api_client = Arc::new(self.clone());

        Clients {
            block: BlockClient::new(api_client.clone()),
            network: NetworkClient::new(api_client.clone()),
            settings: SettingsClient::new(api_client.clone()),
            transaction: TransactionClient::new(api_client.clone()),
            wallet: WalletClient::new(api_client.clone()),
            event: EventClient::new(api_client.clone()),
            address: AddressClient::new(api_client.clone()),
            payment_gateway: PaymentGatewayClient::new(api_client.clone()),
            price_graph: PriceGraphClient::new(api_client.clone()),
            proton_email_address: ProtonEmailAddressClient::new(api_client.clone()),
            exchange_rate: ExchangeRateClient::new(api_client.clone()),
            bitcoin_address: BitcoinAddressClient::new(api_client.clone()),
            contacts: ContactsClient::new(api_client.clone()),
            email_integration: EmailIntegrationClient::new(api_client.clone()),
            invite: InviteClient::new(api_client.clone()),
            discover_content: DiscoverContentClient::new(api_client.clone()),
        }
    }

    /// Performs a http request to authenticate the session used in the api
    /// client. Mutates the underlying session.
    ///
    /// ```rust
    /// use andromeda_api::ProtonWalletApiClient;
    /// let mut api_client = ProtonWalletApiClient::default();
    /// api_client.login("my_username", "my_password");
    /// ```
    pub async fn login(&self, username: &str, password: &str) -> Result<UserData, Error> {
        info!("login start");
        let session = match self.session.clone().auth().login(username, password).await {
            LoginFlow::Ok(session, ..) => session,
            LoginFlow::Failed { session: _, reason: _ } => return Err(Error::LoginError),
            _ => return Err(Error::UnsupportedTwoFactor),
        };
        info!("login successful");

        let req = GET!("/core/v4/users");
        let res = req.send_with(&session).await?;
        let user: CoreAPI::v4::users::GetRes = res.ok()?.into_body_json()?;
        let keysalt_req = GET!("/core/v4/keys/salts");
        let keysalt_res = keysalt_req.send_with(&session).await?;
        let key_salt: CoreAPI::v4::keys::salts::GetRes = keysalt_res.ok()?.into_body_json()?;
        Ok(UserData {
            user: user.user,
            key_salts: key_salt.key_salts,
        })
    }

    /// fork session, client must be authenticated first
    pub async fn fork(&self, client_child: &str, app_version: &str, user_agent: &str) -> Result<ChildSession, Error> {
        use muon::client::flow::WithSelectorFlow;

        let selector = match self
            .session
            .clone()
            .fork(client_child)
            .payload(b"proton wallet fork")
            .send()
            .await
        {
            ForkFlowResult::Success(_, selector) => selector,
            ForkFlowResult::Failure { .. } => return Err(Error::ForkSession),
        };

        let store_env: String = self.env.clone().unwrap_or("atlas".to_string());
        let store = WalletAuthStore::from_env_str(store_env, Arc::new(Mutex::new(Auth::None)));
        let app_spec = App::new(app_version)?.with_user_agent(user_agent);
        let child_client = Client::new(app_spec, store.clone()).await?;
        let child_session = child_client.new_session_without_credentials(()).await?;
        let WithSelectorFlow::Ok(_, payload) = child_session.auth().from_fork().with_selector(selector).await else {
            return Err(Error::ForkAuthSession);
        };
        if payload.as_deref() == Some(b"proton wallet fork".as_ref()) {
            let auth = store.auth.lock().unwrap();
            Ok(ChildSession {
                session_id: auth.uid().unwrap().to_string(),
                access_token: auth.acc_tok().unwrap().to_string(),
                refresh_token: auth.ref_tok().unwrap().to_string(),
                scopes: auth.scopes().unwrap().to_vec(),
            })
        } else {
            Err(Error::Deserialize("Payload not as expected".to_string()))
        }
    }

    /// fork session and get selector, client must be authenticated first
    pub async fn fork_selector(&self, client_child: &str) -> Result<String, Error> {
        match self.session.clone().fork(client_child).send().await {
            ForkFlowResult::Success(_, selector) => Ok(selector),
            ForkFlowResult::Failure { .. } => Err(Error::ForkSession),
        }
    }

    /// Builds a full url from base and endpoint.
    /// If a prefix is set to be used on the apiclient, we prepend it before the
    /// base
    /// TODO:: muon v2 built in prefix we can migrate to that.
    ///  -- use env dirrectly also works like http://localhost/api
    fn build_full_url(&self, base: impl ToString, url: impl ToString) -> String {
        if let Some(prefix) = self.url_prefix.clone() {
            format!("{}/{}/{}", prefix, base.to_string(), url.to_string())
        } else {
            format!("/{}/{}", base.to_string(), url.to_string())
        }
    }

    async fn send(&self, request: ProtonRequest) -> Result<ProtonResponse, MuonError> {
        self.session.clone().send(request).await
    }
}

impl Default for ProtonWalletApiClient {
    /// default Proton Wallet api client. It uses `atlas` env
    fn default() -> Self {
        let default_app = App::new("Other").unwrap();
        let config = ApiConfig {
            spec: (
                default_app.app_version().to_string(),
                default_app.user_agent().to_string(),
            ),
            url_prefix: None,
            env: None,
            store: None,
            auth: None,
            proxy: None,
        };
        Self::from_config(config).unwrap()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn block_on<F>(future: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
    F::Output: Send,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        use tokio::runtime::RuntimeFlavor;

        if handle.runtime_flavor() == RuntimeFlavor::MultiThread {
            return tokio::task::block_in_place(|| handle.block_on(future));
        }

        // Inside a current_thread runtime (e.g. #[tokio::test] without rt-multi-thread):
        // block_in_place panics, so run the future on a separate runtime thread.
        return std::thread::spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create tokio runtime")
                .block_on(future)
        })
        .join()
        .expect("block_on thread panicked");
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(future)
}

#[cfg(target_arch = "wasm32")]
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return handle.block_on(future);
    }

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
        .block_on(future)
}

/// An unsafe verifier that always makes Accept decisions.
#[derive(Debug)]
pub struct UnSafeVerifier;

impl Verifier for UnSafeVerifier {
    fn verify(&self, _: &Host, _: &TlsCert, _: &[TlsCert]) -> Result<VerifyRes, muon::Error> {
        if cfg!(feature = "allow-dangerous-env") {
            Ok(VerifyRes::Accept)
        } else {
            Ok(VerifyRes::Delegate)
        }
    }
}

#[cfg(test)]
mod block_on_tests {
    use super::block_on;
    use crate::tests::utils::test_spec;
    use crate::{ApiConfig, ProtonWalletApiClient};

    #[test]
    fn without_runtime() {
        assert_eq!(block_on(async { 42 }), 42);
    }

    #[tokio::test]
    async fn inside_current_thread_runtime() {
        assert_eq!(block_on(async { 21 + 21 }), 42);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn inside_multi_thread_runtime() {
        assert_eq!(block_on(async { "ok".to_string() }), "ok");
    }

    #[tokio::test]
    async fn from_config_inside_current_thread_runtime() {
        use wiremock::MockServer;

        let mock_server = MockServer::start().await;
        let server_uri = mock_server.uri();
        let config = ApiConfig {
            spec: test_spec(),
            url_prefix: None,
            env: Some(server_uri.clone()),
            store: None,
            auth: None,
            proxy: None,
        };

        assert!(ProtonWalletApiClient::from_config_async(config).await.is_ok());
        let config = ApiConfig {
            spec: test_spec(),
            url_prefix: None,
            env: Some(server_uri),
            store: None,
            auth: None,
            proxy: None,
        };
        assert!(ProtonWalletApiClient::from_config(config).is_ok());
    }
}
