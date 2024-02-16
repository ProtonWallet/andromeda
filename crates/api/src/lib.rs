use std::sync::Arc;

use async_std::sync::RwLock;
use block::BlockClient;
use env::LocalEnv;
use error::Error;
pub use muon::{
    environment::ApiEnv, session::Session, AccessToken, AppSpec, AuthStore, Product, RefreshToken, Scope, Uid,
};
use muon::{store::SimpleAuthStore, ReqwestTransportFactory};
use network::NetworkClient;
use settings::SettingsClient;
use transaction::TransactionClient;
use wallet::WalletClient;

mod env;

pub mod block;
pub mod error;
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
}

/// Dummy struct to build all Wallet api clients from a single sessions
struct ApiClients(
    BlockClient,
    NetworkClient,
    SettingsClient,
    TransactionClient,
    WalletClient,
);

impl ApiClients {
    pub fn from_session(session: Arc<RwLock<Session>>) -> Self {
        ApiClients(
            BlockClient::new(session.clone()),
            NetworkClient::new(session.clone()),
            SettingsClient::new(session.clone()),
            TransactionClient::new(session.clone()),
            WalletClient::new(session.clone()),
        )
    }
}

/// Mirror from Muon's `AuthAccess` with public fields to be easily from outside
/// of the crate
pub struct AuthData {
    pub uid: Uid,
    pub access: AccessToken,
    pub refresh: RefreshToken,
    pub scopes: Vec<Scope>,
}

/// An API client providing interfaces to send authenticated http requests to
/// Wallet backend
///
/// ```no_run
/// # use andromeda_api::ProtonWalletApiClient;
/// # use muon::{session::Session, AppSpec, SimpleAuthStore};
/// # let app_spec = AppSpec::default();
/// # let auth_store = SimpleAuthStore::new("atlas");
/// # let session = Session::new(auth_store, app_spec).unwrap();
/// let mut api_client = ProtonWalletApiClient::default();
/// api_client.login("pro", "pro").await.unwrap();
///
/// let network = api_client.network.get_network().await.unwrap();
/// println!("network fetch: {:?}", network);
/// ```
pub struct ProtonWalletApiClient {
    session: Arc<RwLock<Session>>,

    pub block: BlockClient,
    pub network: NetworkClient,
    pub settings: SettingsClient,
    pub transaction: TransactionClient,
    pub wallet: WalletClient,
}

impl ProtonWalletApiClient {
    #[cfg(feature = "local")]
    fn session(app_spec: AppSpec, auth_store: SimpleAuthStore) -> Session {
        let transport = ReqwestTransportFactory::new();
        let local_env = LocalEnv::new(None);
        Session::new_dangerous(auth_store, app_spec, transport, local_env).unwrap()
    }

    #[cfg(not(feature = "local"))]
    fn session(app_spec: AppSpec, auth_store: SimpleAuthStore) -> Session {
        Session::new(auth_store, app_spec).unwrap()
    }

    /// Builds a new api client from a session.
    ///
    /// Session can be either authenticated or not, authentication can be later
    /// processed.
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// # use muon::{session::Session, AppSpec, SimpleAuthStore};
    /// # let app_spec = AppSpec::default();
    /// # let auth_store = SimpleAuthStore::new("atlas");
    /// let session = Session::new(auth_store, app_spec).unwrap();
    ///
    /// let api_client = ProtonWalletApiClient::from_session(session);
    /// ```
    pub fn from_session(session: Session) -> Self {
        let session = Arc::new(RwLock::new(session));

        let ApiClients(block, network, settings, transaction, wallet) = ApiClients::from_session(session.clone());

        Self {
            session,

            block,
            network,
            settings,
            transaction,
            wallet,
        }
    }

    /// Builds a new api client from user's access token, refresh token, uid and
    /// scope
    ///
    /// ```rust
    /// # use andromeda_api::{ProtonWalletApiClient, AuthData};
    /// # use muon::{AccessToken, RefreshToken, Scopes, Uid};
    /// let auth = AuthData {
    ///     uid: Uid::from("uid....".to_string()),
    ///     access: AccessToken::from("access....".to_string()),
    ///     refresh: RefreshToken::from("refresh....".to_string()),
    ///     scopes: Scopes::from(Vec::<String>::new()),
    /// };
    ///
    /// let api_client = ProtonWalletApiClient::from_auth(auth);
    /// ```
    pub fn from_auth(auth: AuthData) -> Result<Self, Error> {
        let app_spec = WalletAppSpec::new().inner();

        let mut auth_store = SimpleAuthStore::new("local");
        auth_store.set_auth(auth.uid, auth.refresh, auth.access, auth.scopes);

        let session = Self::session(app_spec, auth_store);

        Ok(Self::from_session(session))
    }

    /// Performs an http request to authenticate the session used in the api
    /// client. Mutates the underlying session.
    ///
    /// ```rust
    /// # use andromeda_api::ProtonWalletApiClient;
    /// # use muon::{session::Session, AppSpec, SimpleAuthStore};
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

        let session = Self::session(app_spec, auth_store);

        Self::from_session(session)
    }
}
