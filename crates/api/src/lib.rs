use std::sync::Arc;

use address::AddressClient;
use async_std::sync::RwLock;
use block::BlockClient;
#[cfg(feature = "local")]
use env::LocalEnv;
use error::Error;
use exchange_rate::ExchangeRateClient;

pub use muon::{
    environment::ApiEnv, request::Error as MuonError, session::Session, store::SimpleAuthStore, AccessToken, AppSpec,
    AuthData, AuthStore, Product, RefreshToken, ReqwestTransportFactory, Scope, Uid,
};
use network::NetworkClient;
use settings::SettingsClient;
use transaction::TransactionClient;
use wallet::WalletClient;

mod env;

pub mod address;
pub mod block;
pub mod error;
pub mod exchange_rate;
pub mod network;
pub mod settings;
pub mod transaction;
pub mod wallet;

// TODO: make this private
pub mod utils;

#[cfg(feature = "local")]
pub const BASE_WALLET_API_V1: &str = "/api/wallet/v1";
#[cfg(not(feature = "local"))]
pub const BASE_WALLET_API_V1: &str = "/wallet/v1";

struct WalletAppSpec(AppSpec);

impl WalletAppSpec {
    pub fn new() -> Self {
        let app_spec = AppSpec::new(
            // TODO: change that to Wallet when added to `Product` enum
            Product::Unspecified,
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
        // TODO: change that to Wallet when added to Product enum above
        // Product::Unspecified
        let app_spec = AppSpec::new(Product::Unspecified, app_version, user_agent);
        WalletAppSpec(app_spec)
    }
}

/// Dummy struct to build all Wallet api clients from a single sessions
struct ApiClients(
    BlockClient,
    NetworkClient,
    SettingsClient,
    TransactionClient,
    WalletClient,
    AddressClient,
    ExchangeRateClient,
);

impl ApiClients {
    pub fn from_session(session: Arc<RwLock<Session>>) -> Self {
        ApiClients(
            BlockClient::new(session.clone()),
            NetworkClient::new(session.clone()),
            SettingsClient::new(session.clone()),
            TransactionClient::new(session.clone()),
            WalletClient::new(session.clone()),
            AddressClient::new(session.clone()),
            ExchangeRateClient::new(session.clone()),
        )
    }
}

/// An API client providing interfaces to send authenticated http requests to
/// Wallet backend
///
/// ```no_run
/// # use andromeda_api::ProtonWalletApiClient;
/// # use muon::{session::Session, AppSpec, store::SimpleAuthStore};
/// # let app_spec = AppSpec::default();
/// # let auth_store = SimpleAuthStore::new("atlas");
/// # let session = Session::new(auth_store, app_spec).unwrap();
/// # tokio_test::block_on(async {
/// let mut api_client = ProtonWalletApiClient::default();
/// api_client.login("pro", "pro").await.unwrap();
///
/// let network = api_client.network.get_network().await.unwrap();
/// println!("network fetch: {:?}", network);
/// # })
/// ```
pub struct ProtonWalletApiClient {
    session: Arc<RwLock<Session>>,

    pub block: BlockClient,
    pub network: NetworkClient,
    pub settings: SettingsClient,
    pub transaction: TransactionClient,
    pub wallet: WalletClient,
    pub address: AddressClient,
    pub exchange_rate: ExchangeRateClient,
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
    /// The env for the api client
    pub env: Option<E>,
}

impl ProtonWalletApiClient {
    /// Builds a new api client from a config struct
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// let api_client = ProtonWalletApiClient::from_config("android-wallet/1.0.0".to_string(), "ProtonWallet/plus-agent-details".to_string());
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

        #[cfg(feature = "local")]
        let session = {
            if config.env.is_some() {
                let transport = ReqwestTransportFactory::new();
                Session::new_dangerous(auth_store, app_spec.inner(), transport, config.env.unwrap()).unwrap()
            } else {
                Session::new(auth_store, app_spec.inner()).unwrap()
            }
        };

        #[cfg(not(feature = "local"))]
        let session = Session::new(auth_store, app_spec.inner()).unwrap();

        Self::from_session(session)
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

        let session = Session::new(auth_store, app_spec).unwrap();

        Self::from_session(session)
    }

    /// Builds a new api client from a session.
    ///
    /// Session can be either authenticated or not, authentication can be later
    /// processed.
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// # use muon::{session::Session, AppSpec, store::SimpleAuthStore};
    /// # let app_spec = AppSpec::default();
    /// # let auth_store = SimpleAuthStore::new("atlas");
    /// let session = Session::new(auth_store, app_spec).unwrap();
    ///
    /// let api_client = ProtonWalletApiClient::from_session(session);
    /// ```
    pub fn from_session(session: Session) -> Self {
        let session = Arc::new(RwLock::new(session));

        let ApiClients(block, network, settings, transaction, wallet, address, exchange_rate) =
            ApiClients::from_session(session.clone());

        Self {
            session,

            block,
            network,
            settings,
            transaction,
            wallet,
            address,
            exchange_rate,
        }
    }

    /// Builds a new api client from user's access token, refresh token, uid and
    /// scope
    ///
    /// ```rust
    /// # use andromeda_api::{ProtonWalletApiClient, AuthData};
    /// # use muon::{AccessToken, RefreshToken, Uid};
    /// let auth = AuthData {
    ///     uid: Uid::from("uid....".to_string()),
    ///     access: AccessToken::from("access....".to_string()),
    ///     refresh: RefreshToken::from("refresh....".to_string()),
    ///     scopes: Vec::new(),
    /// };
    ///
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

        Ok(Self::from_session(session))
    }

    /// Performs an http request to authenticate the session used in the api
    /// client. Mutates the underlying session.
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// # use muon::{session::Session, AppSpec, store::SimpleAuthStore};
    /// # let app_spec = AppSpec::default();
    /// # let auth_store = SimpleAuthStore::new("atlas");
    /// # let session = Session::new(auth_store, app_spec).unwrap();
    /// let mut api_client = ProtonWalletApiClient::from_session(session);
    /// api_client.login("my_username", "my_password");
    /// ```
    pub async fn login(&mut self, username: &str, password: &str) -> Result<(), Error> {
        self.session
            .write()
            .await
            .authenticate(username, password)
            .await
            .map_err(|e| e.into())?;

        Ok(())
    }

    pub fn get_session(&self) -> Arc<RwLock<Session>> {
        self.session.clone()
    }
}

impl Default for ProtonWalletApiClient {
    fn default() -> Self {
        let app_spec = WalletAppSpec::new().inner();
        let auth_store = SimpleAuthStore::new("atlas");

        let session = Session::new(auth_store, app_spec).unwrap();

        Self::from_session(session)
    }
}
