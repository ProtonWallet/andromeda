use proton_wallet_common::wallet::{Wallet, WalletConfig};
use wasm_bindgen::prelude::*;

use crate::{
    account::WasmSupportedBIPs,
    error::DetailledWasmError,
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
    ) -> Result<WasmWallet, DetailledWasmError> {
        let wallet = Wallet::new(bip39_mnemonic, bip38_passphrase, config.into()).map_err(|e| e.into())?;

        Ok(Self { inner: wallet })
    }

    #[wasm_bindgen]
    pub async fn add_account(&mut self, bip: WasmSupportedBIPs, account_index: u32) {
        self.inner.add_account(bip.into(), account_index);
    }

    #[wasm_bindgen]
    pub async fn get_balance(&self) -> Result<WasmBalance, DetailledWasmError> {
        let balance = self.inner.get_balance().map_err(|e| e.into())?;
        Ok(balance.into())
    }

    #[wasm_bindgen]
    pub fn get_fingerprint(&self) -> String {
        self.inner.get_fingerprint()
    }
}
