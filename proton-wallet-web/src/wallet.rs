use proton_wallet_common::{wallet::WalletConfig, Wallet};
use wasm_bindgen::prelude::*;

use crate::{
    descriptor::WasmSupportedBIPs,
    error::WasmError,
    types::{balance::WasmBalance, defined::WasmNetwork},
};

#[wasm_bindgen]
pub struct WasmWallet {
    inner: Wallet,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmWalletConfig {
    pub network: WasmNetwork,
}

impl Into<WalletConfig> for WasmWalletConfig {
    fn into(self) -> WalletConfig {
        WalletConfig {
            network: self.network.into(),
        }
    }
}

#[wasm_bindgen]
impl WasmWalletConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(network: Option<WasmNetwork>) -> Self {
        Self {
            network: match network {
                Some(network) => network,
                None => WasmNetwork::Bitcoin,
            },
        }
    }
}

#[wasm_bindgen]
impl WasmWallet {
    // pub fn new_no_persist(descriptor: WasmDescriptor, change_descriptor: Option<WasmDescriptor>, network: WasmNetwork) -> Result<WasmWallet, JsValue> {
    //     let descriptor = descriptor.as_string_private();
    //     let change_descriptor = change_descriptor.map(|d| d.as_string_private());
    //     let net: BdkNetwork = network.into();
    //     let wallet: BdkWallet = BdkWallet::new_no_persist(&descriptor, change_descriptor.as_ref(), net).unwrap();

    //     Ok(WasmWallet {
    //         inner: wallet,
    //     })
    // }

    #[wasm_bindgen(constructor)]
    pub fn new(
        bip39_mnemonic: String,
        bip38_passphrase: Option<String>,
        config: WasmWalletConfig,
    ) -> Result<WasmWallet, WasmError> {
        let wallet = Wallet::new(bip39_mnemonic, bip38_passphrase, config.into()).map_err(|e| e.into())?;

        Ok(Self { inner: wallet })
    }

    #[wasm_bindgen]
    pub async fn add_account(&mut self, bip: WasmSupportedBIPs, account_index: u32) {
        self.inner.add_account(bip.into(), account_index);
    }

    #[wasm_bindgen]
    pub async fn get_balance(self) -> Result<WasmBalance, WasmError> {
        let balance = self.inner.get_balance().await.map_err(|e| e.into())?;
        Ok(balance.into())
    }

    // pub fn get_address(&self, address_index: WasmAddressIndex) -> WasmAddressInfo {
    //     let addressIdex = address_index.into();
    //     self.inner.get_address(addressIdex)
    //     // Assuming get_address returns AddressInfo
    // }

    // pub fn get_internal_address(&self, address_index: AddressIndex) -> AddressInfo {
    //     self.inner.get_internal_address(address_index)
    //     // Assuming get_internal_address returns AddressInfo
    // }

    // pub fn network(&self) -> Network {
    //     self.inner.network()
    //     // Assuming network returns Network
    // }

    // pub fn get_balance(&self) -> Balance {
    //     self.inner.get_balance()
    //     // Assuming get_balance returns Balance
    // }

    // pub fn is_mine(&self, script: Script) -> bool {
    //     self.inner.is_mine(script)
    //     // Assuming is_mine returns a bool
    // }

    // pub fn apply_update(&self, update: Update) -> Result<(), JsValue> {
    //     self.inner.apply_update(update)
    //         .map_err(|e| JsValue::from_str(&e.to_string()))
    //     // Assuming apply_update can throw a BdkError
    // }
}
