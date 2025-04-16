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
use muon::client::flow::LoginExtraInfo;
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
    App, Client, Error as MuonError, ProtonRequest, ProtonResponse, GET,
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
    session: Client,
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
    pub store: Option<DynStore>,
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
    /// Builds a new api client from a config struct
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

        #[cfg(not(target_arch = "wasm32"))]
        let mut builder = Client::builder(app, store);
        #[cfg(target_arch = "wasm32")]
        let builder = Client::builder(app, store);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(proxy) = config.proxy {
            if let Ok(host) = Host::direct(proxy.host) {
                builder = builder.verifier(UnSafeVerifier);
                builder = builder.proxy(ConstProxy::new(Endpoint::new(Scheme::Http, host, proxy.port)));
            }
        }
        let session = builder.build()?;

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
        let extra_info = LoginExtraInfo::builder().build();
        let c = match self
            .session
            .clone()
            .auth()
            .login_with_extra(username, password, extra_info)
            .await
        {
            LoginFlow::Ok(c, ..) => Ok(c),
            LoginFlow::Failed { client: _, reason: _ } => Err(Error::LoginError),
            _ => Err(Error::UnsupportedTwoFactor),
        }?;
        info!("login successful");

        let req = GET!("/core/v4/users");
        let res = req.send_with(&c).await?;
        let user: CoreAPI::v4::users::GetRes = res.ok()?.into_body_json()?;
        let keysalt_req = GET!("/core/v4/keys/salts");
        let keysalt_res = keysalt_req.send_with(&c).await?;
        let key_salt: CoreAPI::v4::keys::salts::GetRes = keysalt_res.ok()?.into_body_json()?;
        Ok(UserData {
            user: user.user,
            key_salts: key_salt.key_salts,
        }) // later return the user
    }

    /// fork session, client must be authenticated first
    pub async fn fork(&self, client_child: &str, app_version: &str, user_agent: &str) -> Result<ChildSession, Error> {
        use muon::client::flow::WithSelectorFlow;

        // Fork the session
        let ForkFlowResult::Success(_client, selector) = self
            .session
            .clone()
            .fork(client_child)
            .payload(b"proton wallet fork")
            .send()
            .await
        else {
            return Err(Error::ForkSession);
        };

        // Create a new client.
        let store_env: String = self.env.clone().unwrap_or("atlas".to_string());
        let store = WalletAuthStore::from_env_str(store_env, Arc::new(Mutex::new(Auth::None)));
        let app_spec = App::new(app_version)?.with_user_agent(user_agent);
        let child = Client::new(app_spec, store.clone())?;
        // Authenticate the child client via the fork.
        let WithSelectorFlow::Ok(_, payload) = child.auth().from_fork().with_selector(selector).await else {
            return Err(Error::ForkAuthSession);
        };
        // The payload is the data sent by the parent client.
        if payload.as_deref() == Some(b"proton wallet fork".as_ref()) {
            let auth = store.auth.lock().unwrap();
            Ok(ChildSession {
                session_id: auth.uid().unwrap().to_string(),
                access_token: auth.acc_tok().unwrap().to_string(),
                refresh_token: auth.ref_tok().unwrap().to_string(),
                scopes: auth.scopes().unwrap().to_vec(),
            })
        } else {
            // Change to our error type
            Err(Error::Deserialize("Payload not as expected".to_string()))
        }
    }

    /// fork session and get selector, client must be authenticated first
    pub async fn fork_selector(&self, client_child: &str) -> Result<String, Error> {
        let ForkFlowResult::Success(_client, selector) = self.session.clone().fork(client_child).send().await else {
            return Err(Error::ForkSession);
        };
        Ok(selector)
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
