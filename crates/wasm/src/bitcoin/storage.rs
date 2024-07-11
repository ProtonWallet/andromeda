use andromeda_bitcoin::{
    error::Error,
    storage::{ChangeSet, WalletStore, WalletStoreFactory},
    Append,
};
use anyhow::anyhow;

#[derive(Clone, Debug)]
pub struct WebOnchainStore {
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

impl WebOnchainStore {
    pub fn new(key: String) -> Self {
        Self {
            changeset_key: format!("{}_{}", CHANGESET_KEY_BASE, key),
        }
    }
}

impl WalletStore for WebOnchainStore {
    fn read(&self) -> Result<Option<ChangeSet>, Error> {
        let local_storage = get_storage().ok();

        if let Some(local_storage) = local_storage {
            let serialized = local_storage.get_item(&self.changeset_key).ok().flatten();

            if let Some(serialized) = serialized {
                return Ok(serde_json::from_str(&serialized).ok());
            }
        }

        Ok(None)
    }

    fn write(&self, new_changeset: &ChangeSet) -> Result<(), Error> {
        let mut prev_changeset = self.read()?.clone().unwrap_or_default();
        prev_changeset.append(new_changeset.clone());

        let serialized =
            serde_json::to_string(&prev_changeset).map_err(|_| anyhow!("Cannot serialize persisted data"))?;

        let local_storage = get_storage().ok();
        if let Some(local_storage) = local_storage {
            local_storage
                .set(&self.changeset_key, &serialized)
                .map_err(|_| anyhow!("Cannot persist data"))?;
        }

        return Ok(());
    }

    fn clear(&self) -> Result<(), Error> {
        let local_storage = get_storage().ok();

        if let Some(local_storage) = local_storage {
            local_storage
                .delete(&self.changeset_key)
                .map_err(|_| anyhow!("Cannot delete data"))?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct WebOnchainStoreFactory();

impl WebOnchainStoreFactory {
    pub fn new() -> Self {
        Self()
    }
}

impl WalletStoreFactory<WebOnchainStore> for WebOnchainStoreFactory {
    fn build(self, key: String) -> WebOnchainStore {
        WebOnchainStore::new(key)
    }
}
