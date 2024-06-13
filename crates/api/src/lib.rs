use core::ApiClient;
use std::sync::{Arc, Mutex};

use address::AddressClient;
use async_std::sync::RwLock;
use bitcoin_address::BitcoinAddressClient;
use block::BlockClient;
use contacts::ContactsClient;
use email_integration::EmailIntegrationClient;
use error::Error;
use event::EventClient;
use exchange_rate::ExchangeRateClient;
use invite::InviteClient;
use log::info;
use muon::ParseAppVersionErr;
pub use muon::{
    flow::LoginFlow,
    http::{HttpReq as ProtonRequest, HttpReqExt, HttpRes as ProtonResponse},
    rest::core as CoreAPI,
    App, Auth, Client, Env, EnvId, Error as MuonError, Product, Store, StoreReadErr, StoreWriteErr, Tokens, GET,
};
use network::NetworkClient;
use proton_email_address::ProtonEmailAddressClient;
pub use proton_users::ProtonUsersClient;
use settings::SettingsClient;
use transaction::TransactionClient;
use wallet::WalletClient;

/// muon removed this expose. enable later
// #[cfg(target_os = "android")]
// pub use {
//     muon::tls::java_init as init_android,
//     muon::tls::{
//         objects::{JClass, JObject},
//         sys::jboolean,
//         JNIEnv,
//     },
// };
pub use crate::{
    core::WalletAuthStore,
    proton_users::{ChildSession, UserData},
};

#[cfg(feature = "quark")]
pub mod proton_quark;

#[cfg(test)]
mod tests;

pub mod address;
pub mod bitcoin_address;
pub mod block;
pub mod contacts;
pub mod email_integration;
pub mod error;
pub mod event;
pub mod exchange_rate;
pub mod invite;
pub mod network;
pub mod payment_gateway;
pub mod proton_email_address;
pub mod proton_users;
pub mod settings;
pub mod transaction;
pub mod wallet;

pub mod core;

pub const BASE_WALLET_API_V1: &str = "/wallet/v1";
pub const BASE_CORE_API_V4: &str = "/core/v4";
pub const BASE_CORE_API_V5: &str = "/core/v5";
pub const BASE_CONTACTS_API_V4: &str = "/contacts/v4";

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
    session: Arc<RwLock<Client>>,
    url_prefix: Option<String>,
}

#[derive(Debug)]
pub struct ApiConfig {
    /// A tupple composed of `app_version` and `user_agent`
    pub spec: Option<(String, String)>,
    /// The api client initial auth data
    pub auth: Option<Auth>,
    /// An optional prefix to use on every api endpoint call
    pub url_prefix: Option<String>,
    /// The env for the api client
    /// could be [altas, prod, or rul link]
    pub env: Option<String>,
}

pub struct Clients {
    pub block: BlockClient,
    pub network: NetworkClient,
    pub settings: SettingsClient,
    pub transaction: TransactionClient,
    pub wallet: WalletClient,
    pub event: EventClient,
    pub address: AddressClient,
    pub proton_email_address: ProtonEmailAddressClient,
    pub exchange_rate: ExchangeRateClient,
    pub bitcoin_address: BitcoinAddressClient,
    pub contacts: ContactsClient,
    pub email_integration: EmailIntegrationClient,
    pub invite: InviteClient,
}

impl ProtonWalletApiClient {
    /// Builds a new api client from a config struct
    ///
    /// env could be custmized url like localhost or 127.0.0.1 it reqires
    /// `allow-dangerous-env` feature to be enabled
    ///
    /// ```rust
    /// use andromeda_api::{ProtonWalletApiClient, ApiConfig, WalletAuthStore};
    /// use muon::{Tokens, Auth};
    /// let auth = Auth::internal("uid", Tokens::access("acc_tok", "ref_tok", ["scopes"]));
    /// let config = ApiConfig {
    ///     spec: Some((String::from("android-wallet/1.0.0"), String::from("ProtonWallet/plus-agent-details"))),
    ///     auth: Some(auth),
    ///     env: Some("atlas".to_string()),
    ///     url_prefix: None,
    /// };
    /// let api_client = ProtonWalletApiClient::from_config(config);
    /// ```
    pub fn from_config(config: ApiConfig) -> Result<Self, Error> {
        let auth = config.auth.unwrap_or(Auth::None);
        let env: String = config.env.unwrap_or("atlas".to_string());
        let auth_store = WalletAuthStore::from_env_str(env, Arc::new(Mutex::new(auth)));
        let app_spec: Result<App, ParseAppVersionErr> = if let Some((app_version, user_agent)) = config.spec {
            Ok(App::new(app_version)?.with_user_agent(user_agent))
        } else {
            Ok(App::default())
        };
        let session = Client::new(app_spec?, auth_store)?;
        Ok(Self::from_session(session, config.url_prefix))
    }

    /// Builds a new api client from a wallet version and a user agent
    ///
    /// ```rust
    /// use andromeda_api::{ProtonWalletApiClient, WalletAuthStore};
    /// let auth_store = WalletAuthStore::default();
    /// let api_client = ProtonWalletApiClient::from_version("android-wallet/1.0.0".to_string(), "ProtonWallet/plus-agent-details".to_string(), auth_store);
    /// ```
    pub fn from_version(app_version: String, user_agent: String, store: impl Store) -> Result<Self, Error> {
        let app_spec = App::new(app_version)?.with_user_agent(user_agent);
        let session = Client::new(app_spec, store)?;
        Ok(Self::from_session(session, None))
    }

    /// Builds a new api client from a session.
    ///
    /// Session can be either authenticated or not, authentication can be later
    /// processed.
    ///
    /// ```rust
    /// use andromeda_api::ProtonWalletApiClient;
    /// use muon::{App, Client};
    /// use andromeda_api::WalletAuthStore;
    /// let app_spec = App::default();
    /// let auth_store = WalletAuthStore::atlas(None);
    /// let session = Client::new(app_spec, auth_store).unwrap();
    /// let api_client = ProtonWalletApiClient::from_session(session, None);
    /// ```
    pub fn from_session(session: Client, url_prefix: Option<String>) -> Self {
        let session = Arc::new(RwLock::new(session));

        Self {
            session: session.clone(),
            url_prefix,
        }
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
            proton_email_address: ProtonEmailAddressClient::new(api_client.clone()),
            exchange_rate: ExchangeRateClient::new(api_client.clone()),
            bitcoin_address: BitcoinAddressClient::new(api_client.clone()),
            contacts: ContactsClient::new(api_client.clone()),
            email_integration: EmailIntegrationClient::new(api_client.clone()),
            invite: InviteClient::new(api_client.clone()),
        }
    }

    /// Performs an http request to authenticate the session used in the api
    /// client. Mutates the underlying session.
    ///
    /// ```rust
    /// use andromeda_api::ProtonWalletApiClient;
    /// use muon::{App, Client};
    /// use andromeda_api::WalletAuthStore;
    /// let app_spec = App::default();
    /// let auth_store = WalletAuthStore::atlas(None);
    /// let session = Client::new(app_spec, auth_store).unwrap();
    /// let mut api_client = ProtonWalletApiClient::from_session(session, None);
    /// api_client.login("my_username", "my_password");
    /// ```
    pub async fn login(&self, username: &str, password: &str) -> Result<UserData, Error> {
        let session_guard = self.session.write().await;
        info!("login start");
        let client = session_guard.clone();
        let LoginFlow::Ok(c) = client.auth().login(username, password).await? else {
            panic!("unexpected auth flow");
        };
        info!("login successful");

        let req = GET!("/core/v4/users");
        let res = req.send_with(&c).await?;
        info!("Getting response");
        let user: CoreAPI::v4::users::GetRes = res.ok()?.into_body_json()?;
        info!("User: {:?}", user);
        let keysalt_req = GET!("/core/v4/keys/salts");
        let keysalt_res = keysalt_req.send_with(&c).await?;
        info!("Getting response");
        let key_salt: CoreAPI::v4::keys::salts::GetRes = keysalt_res.ok()?.into_body_json()?;
        info!("KeySalt: {:?}", key_salt);

        Ok(UserData {
            user: user.user,
            key_salts: key_salt.key_salts,
        }) // later return the user
    }

    pub async fn fork(&self) -> Result<ChildSession, Error> {
        use muon::flow::WithSelectorFlow;

        let session_guard = self.session.write().await;
        let client = session_guard.clone();

        // Fork the session
        let selector = client.fork("ios-wallet").payload(b"hello world").send().await?;
        // Create a new client.
        let store = WalletAuthStore::prod();
        let app_version = "ios-wallet@1.0.0";
        let user_agentuser_agent = "ProtonWallet/1.0.0 (iOS/17.4; arm64)";
        let app_spec = App::new(app_version)?.with_user_agent(user_agentuser_agent);
        let child = Client::new(app_spec, store.clone())?;
        // Authenticate the child client via the fork.
        let WithSelectorFlow::Ok(_, payload) = child.auth().from_fork().with_selector(selector).await?;
        // The payload is the data sent by the parent client.
        if payload.as_deref() == Some(b"hello world".as_ref()) {
            let auth = store.auth.lock().unwrap();
            Ok(ChildSession {
                session_id: auth.uid().unwrap().to_string(),
                access_token: auth.acc_tok().unwrap().to_string(),
                refresh_token: auth.ref_tok().unwrap().to_string(),
                scopes: auth.scopes().unwrap().to_vec(),
            })
        } else {
            // Change to our error type
            Err(Error::DeserializeErr("Payload not as expected".to_string()))
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
            format!("{}/{}", base.to_string(), url.to_string())
        }
    }

    async fn send(&self, request: ProtonRequest) -> Result<ProtonResponse, MuonError> {
        self.session.read().await.send(request).await
    }
}

impl Default for ProtonWalletApiClient {
    /// default Proton Wallet api client. It uses `atlas` env
    fn default() -> Self {
        let app_spec = App::default();
        let auth_store = WalletAuthStore::atlas(None);
        let session = Client::new(app_spec, auth_store).unwrap();
        return Self::from_session(session, None);
    }
}
