use std::sync::{Arc, Mutex};

use cfg_if::cfg_if;
use muon::{common::IntoDyn, Auth, Env, EnvId, Store, StoreReadErr, StoreWriteErr};
#[cfg(feature = "allow-dangerous-env")]
use {
    muon::{
        common::{Endpoint, Server},
        tls::TlsPinSet,
        AppVersion,
    },
    std::str::FromStr,
};

#[derive(Debug, Clone)]
pub struct WalletAuthStore {
    pub env: EnvId,
    pub auth: Arc<Mutex<Auth>>,
}

impl Default for WalletAuthStore {
    fn default() -> Self {
        Self::prod()
    }
}

impl WalletAuthStore {
    pub fn from_env_str(env: String, auth: Arc<Mutex<Auth>>) -> Self {
        if let Ok(env) = env.parse() {
            Self { env, auth }
        } else {
            Self::custom_env(env, auth)
        }
    }

    /// Create a new store for the given environment.
    pub fn new(env: EnvId) -> Self {
        Self {
            env,
            auth: Arc::new(Mutex::new(Auth::None)),
        }
    }

    /// Create a new prod store.
    pub fn prod() -> Self {
        Self::new(EnvId::Prod)
    }

    /// Create a new atlas store.
    pub fn atlas(option: Option<String>) -> Self {
        Self::new(EnvId::Atlas(option))
    }

    /// Create a new store for a custom environment.
    pub fn custom(env: impl Env) -> Self {
        Self::new(EnvId::Custom(env.into_dyn()))
    }
}

impl Store for WalletAuthStore {
    fn env(&self) -> EnvId {
        self.env.clone()
    }

    fn get_auth(&self) -> Result<Auth, StoreReadErr> {
        let auth = self.auth.lock().unwrap().clone();
        Ok(auth.clone())
    }

    fn set_auth(&mut self, auth: Auth) -> Result<Auth, StoreWriteErr> {
        let mut old_auth = self.auth.lock().unwrap();
        *old_auth = auth.clone();
        Ok(auth)
    }
}

cfg_if! {
    if #[cfg(feature = "allow-dangerous-env")] {
        struct WalletCustomEnv {
            inner: String,
        }
        /// Implement [`Env`] to specify the servers for the custom environment.
        impl Env for WalletCustomEnv {
            fn servers(&self, _: &AppVersion) -> Vec<Server> {
                let endpoint = Endpoint::from_str(self.inner.as_str()).expect("Invalid server address");
                let path = String::from("/");

                vec![Server{endpoint, path}]
            }

            fn pins(&self, _: &Server) -> Option<TlsPinSet> {
                None
            }
        }

        impl WalletAuthStore {
            fn custom_env(env: String, auth: Arc<Mutex<Auth>>) -> Self {
                Self {
                    env: EnvId::Custom(WalletCustomEnv{inner: env}.into_dyn()),
                    auth,
                }
            }
        }
    } else {
        impl WalletAuthStore {
            fn custom_env(_env: String, _auth: Arc<Mutex<Auth>>) -> Self {
                panic!("the `allow-dangerous-env` feature must be enabled");
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use muon::EnvId;

    #[test]
    fn test_build_wallet_auth_store() {
        use std::sync::{Arc, Mutex};

        use muon::Auth;

        use crate::WalletAuthStore;

        let auth = Arc::new(Mutex::new(Auth::None));
        let store = WalletAuthStore::from_env_str("prod".to_string(), auth.clone());
        assert!(matches!(store.env, EnvId::Prod));
        let store = WalletAuthStore::from_env_str("atlas".to_string(), auth.clone());
        assert!(matches!(store.env, EnvId::Atlas(None)));
        let store = WalletAuthStore::from_env_str("atlas:scientist".to_string(), auth.clone());
        assert!(matches!(store.env, EnvId::Atlas(Some(name)) if name == "scientist"));

        // add more tests for customized envs.
    }

    #[test]
    fn test_parse_env_id() {
        let env: EnvId = "prod".parse().unwrap();
        assert!(matches!(env, EnvId::Prod));

        let env: EnvId = "atlas".parse().unwrap();
        assert!(matches!(env, EnvId::Atlas(None)));

        let env: EnvId = "atlas:scientist".parse().unwrap();
        assert!(matches!(env, EnvId::Atlas(Some(name)) if name == "scientist"));
    }
}
