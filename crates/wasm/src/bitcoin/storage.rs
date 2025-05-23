use std::sync::Arc;

use andromeda_bitcoin::{
    error::Error,
    storage::{ChangeSet, Merge, Storage, WalletPersisterFactory},
};
use anyhow::anyhow;

#[derive(Clone, Debug)]
pub struct WalletWebPersister {
    changeset_key: String,
}

const CHANGESET_KEY_BASE: &str = "CHANGESET";

fn get_storage() -> Result<web_sys::Storage, js_sys::Error> {
    let window = web_sys::window().ok_or(js_sys::Error::new("No window in context"))?;
    let local_storage = window
        .local_storage()
        .map_err(|_| js_sys::Error::new("Cannot get local storage"))?
        .ok_or(js_sys::Error::new("No local storage found"))?;

    Ok(local_storage)
}

impl WalletWebPersister {
    pub fn new(key: String) -> Self {
        Self {
            changeset_key: format!("{}_{}", CHANGESET_KEY_BASE, key),
        }
    }

    fn get(&self) -> Option<ChangeSet> {
        let local_storage = get_storage().ok();

        if let Some(local_storage) = local_storage {
            let serialized = local_storage.get_item(&self.changeset_key).ok().flatten();

            if let Some(serialized) = serialized {
                return serde_json::from_str(&serialized).ok();
            }
        }

        None
    }

    fn set(&self, changeset: ChangeSet) -> Result<(), Error> {
        let serialized = serde_json::to_string(&changeset).map_err(|_| anyhow!("Cannot serialize persisted data"))?;

        let local_storage = get_storage().ok();
        if let Some(local_storage) = local_storage {
            local_storage
                .set(&self.changeset_key, &serialized)
                .map_err(|_| anyhow!("Cannot persist data"))?;
        }

        Ok(())
    }
}

impl Storage for WalletWebPersister {
    fn initialize(&self) -> Result<ChangeSet, Error> {
        Ok(self.get().unwrap_or_default())
    }

    fn persist(&self, new_changeset: &ChangeSet) -> Result<(), Error> {
        let mut prev_changeset = self.get().unwrap_or_default();
        prev_changeset.merge(new_changeset.clone());

        self.set(prev_changeset)
    }
}

#[derive(Debug, Clone)]
pub struct WalletWebPersisterFactory;
impl WalletPersisterFactory for WalletWebPersisterFactory {
    fn build(self, key: String) -> Arc<dyn Storage> {
        Arc::new(WalletWebPersister::new(key))
    }
}
