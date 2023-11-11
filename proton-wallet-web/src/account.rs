use proton_wallet_common::{
    account::{Account, AccountConfig},
    keys::new_master_private_key,
};

use wasm_bindgen::prelude::*;

use crate::{balance::WasmBalance, defined::WasmNetwork, descriptor::WasmSupportedBIPs, error::WasmError};

#[wasm_bindgen]
pub struct WasmAccount {
    inner: Account,
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
impl WasmAccount {
    #[wasm_bindgen(constructor)]
    pub fn new(
        mnemonic_str: &str,
        passphrase: Option<String>,
        config: WasmAccountConfig,
    ) -> Result<WasmAccount, WasmError> {
        let mprivkey = new_master_private_key(mnemonic_str, passphrase);

        let account = Account::new(mprivkey, config.into()).map_err(|e| e.into())?;

        Ok(Self { inner: account })
    }

    #[wasm_bindgen]
    pub async fn get_balance(self) -> Result<WasmBalance, WasmError> {
        let balance = self.inner.get_balance().await.map_err(|e| e.into())?;
        Ok(balance.into())
    }
}
