use proton_wallet_common::{
    account::{Account, AccountConfig, SupportedBIPs},
    new_master_private_key,
};

use wasm_bindgen::prelude::*;

use crate::{
    error::DetailledWasmError,
    types::{balance::WasmBalance, defined::WasmNetwork},
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
pub struct WasmAccount {
    inner: Account,
}

impl Into<Account> for WasmAccount {
    fn into(self) -> Account {
        self.inner.clone()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmAccountConfig {
    pub bip: WasmSupportedBIPs,
    pub network: WasmNetwork,
    pub account_index: u32,
}

impl Into<AccountConfig> for WasmAccountConfig {
    fn into(self) -> AccountConfig {
        AccountConfig {
            bip: self.bip.into(),
            account_index: self.account_index,
            network: self.network.into(),
        }
    }
}

#[wasm_bindgen]
impl WasmAccountConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(bip: Option<WasmSupportedBIPs>, network: Option<WasmNetwork>, account_index: Option<u32>) -> Self {
        Self {
            bip: match bip {
                Some(bip) => bip,
                None => WasmSupportedBIPs::Bip44,
            },
            network: match network {
                Some(network) => network,
                None => WasmNetwork::Bitcoin,
            },
            account_index: match account_index {
                Some(account_index) => account_index,
                None => 0,
            },
        }
    }
}

#[wasm_bindgen]
impl WasmAccount {
    #[wasm_bindgen(constructor)]
    pub fn new(
        mnemonic_str: &str,
        passphrase: Option<String>,
        config: WasmAccountConfig,
    ) -> Result<WasmAccount, DetailledWasmError> {
        let mprivkey = new_master_private_key(mnemonic_str, passphrase);

        let account = Account::new(mprivkey, config.into()).map_err(|e| e.into())?;

        Ok(Self { inner: account })
    }

    #[wasm_bindgen]
    pub async fn get_balance(self) -> Result<WasmBalance, DetailledWasmError> {
        let balance = self.inner.get_balance().await.map_err(|e| e.into())?;
        Ok(balance.into())
    }
}
