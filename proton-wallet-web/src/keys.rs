use std::sync::Arc;

use proton_wallet_common::{DescriptorPublicKey, DescriptorSecretKey};
use wasm_bindgen::prelude::*;

use crate::{derivation_path::WasmDerivationPath, error::WasmError};

#[wasm_bindgen]
pub struct WasmDescriptorSecretKey {
    inner: DescriptorSecretKey,
}

impl From<WasmDescriptorSecretKey> for DescriptorSecretKey {
    fn from(value: WasmDescriptorSecretKey) -> DescriptorSecretKey {
        value.inner
    }
}

impl Into<WasmDescriptorSecretKey> for DescriptorSecretKey {
    fn into(self) -> WasmDescriptorSecretKey {
        WasmDescriptorSecretKey { inner: self }
    }
}

impl WasmDescriptorSecretKey {
    pub fn from_string(secret_key: &str) -> Result<WasmDescriptorSecretKey, WasmError> {
        DescriptorSecretKey::from_string(secret_key.to_string())
            .map(|pk| WasmDescriptorSecretKey { inner: pk })
            .map_err(|_| WasmError::InvalidSecretKey)
    }

    // Method to derive a new WasmDescriptorPublicKey
    pub fn derive(&self, path: WasmDerivationPath) -> Result<WasmDescriptorSecretKey, WasmError> {
        self.inner
            .derive(path.into())
            .map(|pk| WasmDescriptorSecretKey {
                inner: Arc::into_inner(pk).unwrap(),
            })
            .map_err(|_| WasmError::InvalidDerivationPath)
    }

    // Method to extend the DescriptorPublicKey
    pub fn extend(&self, derivation_path: WasmDerivationPath) -> Result<WasmDescriptorSecretKey, WasmError> {
        self.inner
            .extend(derivation_path.into())
            .map(|pk| WasmDescriptorSecretKey {
                inner: Arc::into_inner(pk).unwrap(),
            })
            .map_err(|_| WasmError::InvalidDerivationPath)
    }

    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }
}

#[wasm_bindgen]
pub struct WasmDescriptorPublicKey {
    inner: DescriptorPublicKey,
}

impl From<WasmDescriptorPublicKey> for DescriptorPublicKey {
    fn from(value: WasmDescriptorPublicKey) -> DescriptorPublicKey {
        value.inner
    }
}

impl Into<WasmDescriptorPublicKey> for DescriptorPublicKey {
    fn into(self) -> WasmDescriptorPublicKey {
        WasmDescriptorPublicKey { inner: self }
    }
}

impl WasmDescriptorPublicKey {
    pub fn from_string(public_key: &str) -> Result<WasmDescriptorPublicKey, WasmError> {
        DescriptorPublicKey::from_string(public_key.to_string())
            .map(|pk| WasmDescriptorPublicKey { inner: pk })
            .map_err(|_| WasmError::InvalidSecretKey)
    }

    // Method to derive a new WasmDescriptorPublicKey
    pub fn derive(&self, path: WasmDerivationPath) -> Result<WasmDescriptorPublicKey, WasmError> {
        self.inner
            .derive(path.into())
            .map(|pk| WasmDescriptorPublicKey {
                inner: Arc::into_inner(pk).unwrap(),
            })
            .map_err(|_| WasmError::InvalidDerivationPath)
    }

    // Method to extend the DescriptorPublicKey
    pub fn extend(&self, derivation_path: WasmDerivationPath) -> Result<WasmDescriptorPublicKey, WasmError> {
        self.inner
            .extend(derivation_path.into())
            .map(|pk| WasmDescriptorPublicKey {
                inner: Arc::into_inner(pk).unwrap(),
            })
            .map_err(|_| WasmError::InvalidDerivationPath)
    }

    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }
}
