use andromeda_bitcoin::{storage::PersistBackendFactory, Append, ChangeSet, PersistBackend};
use anyhow::anyhow;

#[derive(Clone, Debug)]
pub struct OnchainStorage {
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

impl OnchainStorage {
    pub fn new(key: String) -> Self {
        Self {
            changeset_key: format!("{}_{}", CHANGESET_KEY_BASE, key),
        }
    }
}

impl PersistBackend<ChangeSet> for OnchainStorage {
    fn write_changes(&mut self, changeset: &ChangeSet) -> Result<(), anyhow::Error> {
        let mut data = self.load_from_persistence().unwrap_or_default().unwrap_or_default();
        data.append(changeset.clone());

        let serialized = serde_json::to_string(&data).map_err(|_| anyhow!("Cannot serialize persisted data"))?;

        let local_storage = get_storage().ok();
        if local_storage.is_some() {
            local_storage
                .unwrap()
                .set(&self.changeset_key, &serialized)
                .map_err(|_| anyhow!("Cannot persist data"))?;
        }

        return Ok(());
    }

    fn load_from_persistence(&mut self) -> Result<Option<ChangeSet>, anyhow::Error> {
        let local_storage = get_storage().ok();

        match local_storage {
            Some(local_storage) => {
                let serialized = local_storage.get_item(&self.changeset_key).ok().flatten();

                match serialized {
                    Some(serialized) => Ok(serde_json::from_str(&serialized).ok()),
                    _ => Ok(None),
                }
            }
            _ => Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct OnchainStorageFactory();

impl OnchainStorageFactory {
    pub fn new() -> Self {
        Self()
    }
}

impl PersistBackendFactory<OnchainStorage> for OnchainStorageFactory {
    fn build(self, key: String) -> OnchainStorage {
        OnchainStorage::new(key)
    }
}
