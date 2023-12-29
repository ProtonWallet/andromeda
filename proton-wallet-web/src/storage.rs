use proton_wallet_common::{Append, ChangeSet, PersistBackend};
use serde::Serialize;

use crate::error::WasmError;

#[derive(Clone)]
pub struct OnchainStorage {
    changeset_key: String,
}

const CHANGESET_KEY_BASE: &str = "CHANGESET";

fn get_storage() -> Result<web_sys::Storage, WasmError> {
    let window = web_sys::window().ok_or(WasmError::NoWindowContext)?;
    let local_storage = window
        .local_storage()
        .map_err(|_| WasmError::CannotGetLocalStorage)?
        .ok_or(WasmError::CannotGetLocalStorage)?;

    Ok(local_storage)
}

impl OnchainStorage {
    pub fn new(account_id: String) -> Self {
        Self {
            changeset_key: format!("{}_{}", CHANGESET_KEY_BASE, account_id),
        }
    }

    pub fn exists(&self) -> bool {
        get_storage()
            .ok()
            .and_then(|storage| storage.get_item(&self.changeset_key).ok())
            .map_or(false, |item| item.map_or(false, |i| i.is_empty()))
    }
}

impl PersistBackend<ChangeSet> for OnchainStorage {
    type WriteError = WasmError;
    type LoadError = WasmError;

    fn write_changes(&mut self, changeset: &ChangeSet) -> Result<(), Self::WriteError> {
        let local_storage = get_storage()?;

        let mut data = self.load_from_persistence()?;
        data.append(changeset.clone());

        let serialized = serde_json::to_string(&data).map_err(|_| WasmError::CannotSerializePersistedData)?;

        local_storage
            .set(&self.changeset_key, &serialized)
            .map_err(|_| WasmError::CannotPersistData)?;

        return Ok(());
    }

    fn load_from_persistence(&mut self) -> Result<ChangeSet, Self::LoadError> {
        let local_storage = get_storage()?;

        let serialized = local_storage
            .get_item(&self.changeset_key)
            .map_err(|_| WasmError::CannotFindPersistedData)?;

        match serialized {
            Some(serialized) => Ok(serde_json::from_str(&serialized).map_err(|_| WasmError::CannotParsePersistedData)?),
            _ => Ok(ChangeSet::default()),
        }
    }
}
