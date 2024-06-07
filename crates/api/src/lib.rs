#[cfg(not(feature = "allow-dangerous-env"))]
use std::marker::PhantomData;
use std::sync::Arc;

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
pub use muon::{
    environment::ApiEnv, store::SimpleAuthStore, transports::ReqwestTransportFactory, AccessToken, AppSpec, Auth,
    AuthData, AuthStore, Error as MuonError, Product, RefreshToken, Scope, Scopes, Session, Uid,
};
use muon::{ProtonRequest, ProtonResponse, RequestError};
use network::NetworkClient;
use proton_email_address::ProtonEmailAddressClient;
use settings::SettingsClient;
use transaction::TransactionClient;
use wallet::WalletClient;

#[cfg(feature = "local")]
mod env;

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

pub use proton_users::ProtonUsersClient;

mod core;

pub const BASE_WALLET_API_V1: &str = "/wallet/v1";
pub const BASE_CORE_API_V4: &str = "/core/v4";
pub const BASE_CORE_API_V5: &str = "/core/v5";
pub const BASE_CONTACTS_API_V4: &str = "/contacts/v4";

pub struct WalletAppSpec(AppSpec);

impl WalletAppSpec {
    pub fn new() -> Self {
        let app_spec = AppSpec::new(
            Product::Wallet,
            // TODO: change that when Wallet has a version (or provide it through args)
            "Other".to_owned(),
            // TODO: change that by provide user agent when building pw api client
            "None".to_owned(),
        );
        WalletAppSpec(app_spec)
    }

    pub fn inner(&self) -> AppSpec {
        self.0.clone()
    }

    pub fn from_version(app_version: String, user_agent: String) -> Self {
        let app_spec = AppSpec::new(Product::Wallet, app_version, user_agent);
        WalletAppSpec(app_spec)
    }
}

// Default wallet app spec
impl Default for WalletAppSpec {
    fn default() -> Self {
        Self::new()
    }
}

/// An API client providing interfaces to send authenticated http requests to
/// Wallet backend
///
/// ```no_run
/// # use andromeda_api::ProtonWalletApiClient;
/// # use muon::{Session, AppSpec, store::SimpleAuthStore};
/// # let app_spec = AppSpec::default();
/// # let auth_store = SimpleAuthStore::new("atlas");
/// # let session = Session::new(auth_store, app_spec).unwrap();
/// # tokio_test::block_on(async {
/// let mut api_client = ProtonWalletApiClient::default();
/// api_client.login("pro", "pro").await.unwrap();
///
/// let network = api_client.clients().network.get_network().await.unwrap();
/// println!("network fetch: {:?}", network);
/// # })
/// ```
#[derive(Clone)]
pub struct ProtonWalletApiClient {
    session: Arc<RwLock<Session>>,
    url_prefix: Option<String>,
}

#[derive(Debug)]
pub struct ApiConfig<E>
where
    E: ApiEnv,
{
    /// A tupple composed of `app_version` and `user_agent`
    pub spec: Option<(String, String)>,
    /// The api client initial auth data
    pub auth: Option<AuthData>,
    /// An optional prefix to use on every api endpoint call
    pub url_prefix: Option<String>,
    /// The env for the api client
    #[cfg(feature = "allow-dangerous-env")]
    pub env: Option<E>,
    #[cfg(not(feature = "allow-dangerous-env"))]
    pub env: PhantomData<E>,
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
    /// ```rust
    /// # use andromeda_api::{ProtonWalletApiClient, AuthData, ApiConfig};
    /// # use muon::{store::SimpleAuthStore, AccessToken, RefreshToken, Uid, environment::AtlasEnv};
    /// let auth = AuthData::Access(Uid::from("uid....".to_string()), RefreshToken::from("refresh....".to_string()), AccessToken::from("access....".to_string()), Vec::new());
    /// let config = ApiConfig {
    ///     spec: Some((String::from("android-wallet/1.0.0"), String::from("ProtonWallet/plus-agent-details"))),
    ///     auth: Some(auth),
    ///     env: Some(AtlasEnv::new(None)),
    ///     url_prefix: None,
    /// };
    /// let api_client = ProtonWalletApiClient::from_config(config);
    /// ```
    pub fn from_config<E>(config: ApiConfig<E>) -> Self
    where
        E: ApiEnv,
    {
        // TODO: this needs to be fixed -> 'atlas' should not be hardcoded
        let mut auth_store = SimpleAuthStore::new("atlas");

        if config.auth.is_some() {
            match config.auth.unwrap() {
                AuthData::Uid(uid) => {
                    auth_store.set_uid_auth(uid);
                }
                AuthData::Access(uid, refresh, access, scopes) => {
                    auth_store.set_access_auth(uid, refresh, access, scopes);
                }
                _ => {}
            }
        }

        let app_spec = if let Some((app_version, user_agent)) = config.spec {
            WalletAppSpec::from_version(app_version, user_agent)
        } else {
            WalletAppSpec::new()
        };

        let transport = ReqwestTransportFactory::new();

        #[cfg(feature = "allow-dangerous-env")]
        let session = {
            if let Some(env) = config.env {
                Session::new_dangerous(auth_store, app_spec.inner(), transport, env).unwrap()
            } else {
                Session::new_with_transport(auth_store, app_spec.inner(), transport).unwrap()
            }
        };

        #[cfg(not(feature = "allow-dangerous-env"))]
        let session = Session::new_with_transport(auth_store, app_spec.inner(), transport).unwrap();

        Self::from_session(session, config.url_prefix)
    }

    /// Builds a new api client from a wallet version and a user agent
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// let api_client = ProtonWalletApiClient::from_version("android-wallet/1.0.0".to_string(), "ProtonWallet/plus-agent-details".to_string());
    /// ```
    pub fn from_version(app_version: String, user_agent: String) -> Self {
        let app_spec = WalletAppSpec::from_version(app_version, user_agent).inner();
        let auth_store = SimpleAuthStore::new("atlas");
        let transport = ReqwestTransportFactory::new();
        let session = Session::new_with_transport(auth_store, app_spec, transport).unwrap();
        Self::from_session(session, None)
    }

    /// Builds a new api client from a session.
    ///
    /// Session can be either authenticated or not, authentication can be later
    /// processed.
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// # use muon::{Session, AppSpec, store::SimpleAuthStore};
    /// # let app_spec = AppSpec::default();
    /// # let auth_store = SimpleAuthStore::new("atlas");
    /// let session = Session::new(auth_store, app_spec).unwrap();
    ///
    /// let api_client = ProtonWalletApiClient::from_session(session, None);
    /// ```
    pub fn from_session(session: Session, url_prefix: Option<String>) -> Self {
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

    /// Builds a new api client from user's access token, refresh token, uid and
    /// scope
    ///
    /// ```rust
    /// # use muon::{AccessToken, RefreshToken, Uid};
    /// # use andromeda_api::{ProtonWalletApiClient, AuthData};
    /// let auth = AuthData::Access(Uid::from("uid....".to_string()), RefreshToken::from("refresh....".to_string()), AccessToken::from("access....".to_string()), Vec::new());
    /// let api_client = ProtonWalletApiClient::from_auth(auth);
    /// ```
    pub fn from_auth(auth: AuthData) -> Result<Self, Error> {
        let app_spec = WalletAppSpec::new().inner();

        let mut auth_store = SimpleAuthStore::new("atlas");

        match auth {
            AuthData::Uid(uid) => {
                auth_store.set_uid_auth(uid);
            }
            AuthData::Access(uid, refresh, access, scopes) => {
                auth_store.set_access_auth(uid, refresh, access, scopes);
            }
            _ => {}
        }

        let session = Session::new(auth_store, app_spec).unwrap();

        Ok(Self::from_session(session, None))
    }

    /// Builds a new api client from user's access token, refresh token, uid,
    /// scope, wallet version and a user agent
    ///
    /// ```rust
    /// # use muon::{AccessToken, RefreshToken, Uid};
    /// # use andromeda_api::{ProtonWalletApiClient, AuthData};
    /// let auth = AuthData::Access(Uid::from("uid....".to_string()), RefreshToken::from("refresh....".to_string()), AccessToken::from("access....".to_string()), Vec::new());
    /// let api_client = ProtonWalletApiClient::from_auth_with_version(auth, "Other".to_owned(), "None".to_owned(), None);
    /// ```
    pub fn from_auth_with_version(
        auth: AuthData,
        app_version: String,
        user_agent: String,
        env: Option<String>,
    ) -> Result<Self, Error> {
        let app_spec = WalletAppSpec::from_version(app_version, user_agent).inner();
        let auth_store_env = env.unwrap_or("atlas".to_string());
        let mut auth_store = SimpleAuthStore::new(auth_store_env.clone());

        match auth {
            AuthData::Uid(uid) => {
                auth_store.set_uid_auth(uid);
            }
            AuthData::Access(uid, refresh, access, scopes) => {
                auth_store.set_access_auth(uid, refresh, access, scopes);
            }
            _ => {}
        }

        let transport = ReqwestTransportFactory::new();
        let session = Session::new_with_transport(auth_store, app_spec, transport).unwrap();

        Ok(Self::from_session(session, None))
    }

    /// Performs an http request to authenticate the session used in the api
    /// client. Mutates the underlying session.
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// # use muon::{Session, AppSpec, store::SimpleAuthStore};
    /// # let app_spec = AppSpec::default();
    /// # let auth_store = SimpleAuthStore::new("atlas");
    /// # let session = Session::new(auth_store, app_spec).unwrap();
    /// let mut api_client = ProtonWalletApiClient::from_session(session, None);
    /// api_client.login("my_username", "my_password");
    /// ```
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        self.session.write().await.authenticate(username, password).await?;

        Ok(())
    }

    /// Builds a full url from base and endpoint.
    /// If a prefix is set to be used on the apiclient, we prepend it before the
    /// base
    fn build_full_url(&self, base: impl ToString, url: impl ToString) -> String {
        if let Some(prefix) = self.url_prefix.clone() {
            format!("{}/{}/{}", prefix, base.to_string(), url.to_string())
        } else {
            format!("{}/{}", base.to_string(), url.to_string())
        }
    }

    async fn send(&self, request: ProtonRequest) -> Result<ProtonResponse, RequestError> {
        self.session.read().await.bind(request)?.send().await
    }
}

impl Default for ProtonWalletApiClient {
    /// default Proton Wallet api client. It uses `atlas` env
    fn default() -> Self {
        let app_spec = WalletAppSpec::new().inner();
        let auth_store = SimpleAuthStore::new("atlas");

        let session = Session::new(auth_store, app_spec).unwrap();

        Self::from_session(session, None)
    }
}
