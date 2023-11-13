use std::sync::Arc;

use proton_wallet_common::{
    descriptor::{SupportedBIPs, SupportedBIPsPublic},
    Descriptor, DescriptorPublicKey, DescriptorSecretKey,
};
use wasm_bindgen::prelude::*;

use crate::{
    error::WasmError,
    keys::{WasmDescriptorPublicKey, WasmDescriptorSecretKey}, types::defined::{WasmKeychainKind, WasmNetwork},
};

#[wasm_bindgen]
#[derive(Clone)]
pub enum WasmSupportedBIPs {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl Into<SupportedBIPs> for WasmSupportedBIPs {
    fn into(self) -> SupportedBIPs {
        match self {
            WasmSupportedBIPs::Bip44 => SupportedBIPs::Bip44,
            WasmSupportedBIPs::Bip49 => SupportedBIPs::Bip49,
            WasmSupportedBIPs::Bip84 => SupportedBIPs::Bip84,
            WasmSupportedBIPs::Bip86 => SupportedBIPs::Bip86,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum WasmSupportedBIPsPublic {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

// TODO: maybe useless, only need to use SupportedBIPs
impl Into<SupportedBIPsPublic> for WasmSupportedBIPsPublic {
    fn into(self) -> SupportedBIPsPublic {
        match self {
            WasmSupportedBIPsPublic::Bip44 => SupportedBIPsPublic::Bip44,
            WasmSupportedBIPsPublic::Bip49 => SupportedBIPsPublic::Bip49,
            WasmSupportedBIPsPublic::Bip84 => SupportedBIPsPublic::Bip84,
            WasmSupportedBIPsPublic::Bip86 => SupportedBIPsPublic::Bip86,
        }
    }
}

#[wasm_bindgen]
pub struct WasmDescriptor {
    inner: Descriptor,
}

#[wasm_bindgen]
#[derive(Clone)]
pub enum WasmChildNumberType {
    Normal,
    Hardened,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmChildNumber {
    pub kind: WasmChildNumberType,
    pub index: u32,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmDescriptorConfig {
    pub bip: WasmSupportedBIPs,
    pub keychain_kind: WasmKeychainKind,
    pub network: WasmNetwork,
    pub account: WasmChildNumber,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmDescriptorConfigPublic {
    pub bip: WasmSupportedBIPsPublic,
    pub keychain_kind: WasmKeychainKind,
    pub network: WasmNetwork,
    pub account: WasmChildNumber,
}

#[wasm_bindgen]
impl WasmDescriptor {
    #[wasm_bindgen(constructor)]
    pub fn new(descriptor: &str, network: WasmNetwork) -> Result<WasmDescriptor, WasmError> {
        Descriptor::new(descriptor.to_string(), network.into())
            .map(|descriptor| WasmDescriptor { inner: descriptor })
            .map_err(|_| WasmError::InvalidDescriptor)
    }

    pub fn from_secret_key(
        key: WasmDescriptorSecretKey,
        config: WasmDescriptorConfig,
    ) -> Result<WasmDescriptor, WasmError> {
        let secret_key = DescriptorSecretKey::from(key);

        let descriptor = Descriptor::new_bip(
            Arc::new(secret_key),
            config.bip.into(),
            config.keychain_kind.into(),
            config.network.into(),
        );

        Ok(WasmDescriptor { inner: descriptor })
    }

    pub fn from_public_key(
        key: WasmDescriptorPublicKey,
        fingerprint: String,
        config: WasmDescriptorConfigPublic,
    ) -> Result<WasmDescriptor, WasmError> {
        let secret_key = DescriptorPublicKey::from(key);

        let descriptor = Descriptor::new_bip_public(
            Arc::new(secret_key),
            fingerprint,
            config.bip.into(),
            config.keychain_kind.into(),
            config.network.into(),
        );

        Ok(WasmDescriptor { inner: descriptor })
    }

    // Method to get the descriptor as a string
    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }

    // Method to get the private descriptor as a string
    #[wasm_bindgen(js_name = asStringPrivate)]
    pub fn as_string_private(&self) -> String {
        self.inner.as_string_private()
    }
}
