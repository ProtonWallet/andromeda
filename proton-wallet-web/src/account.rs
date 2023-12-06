use std::sync::Mutex;

use proton_wallet_common::{
    account::{Account, AccountConfig, SupportedBIPs},
    DerivableKey, ExtendedKey, ExtendedPrivKey, Mnemonic, MnemonicWithPassphrase,
};

use wasm_bindgen::prelude::*;
use web_sys::console::log_1;

use crate::{
    error::{DetailledWasmError, WasmError},
    types::{
        balance::WasmBalance, defined::WasmNetwork, pagination::WasmPagination, transaction::WasmSimpleTransaction,
    },
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
    inner: Mutex<Account>,
}

impl WasmAccount {
    pub fn into_mutable(&mut self) -> &mut Account {
        self.inner.get_mut().unwrap()
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
        log_1(&"here 1".into());
        let mnemonic = Mnemonic::parse(mnemonic_str).map_err(|_| WasmError::InvalidMnemonic.into())?;
        let passphrase = match passphrase {
            Some(passphrase) => passphrase,
            _ => "".to_string(),
        };

        let config: AccountConfig = config.into();
        let mprivkey = ExtendedPrivKey::new_master(config.network.into(), &mnemonic.to_seed(passphrase))
            .map_err(|_| WasmError::InvalidSeed.into())?;

        log_1(&"here 2".into());

        let account = Account::new(mprivkey, config.into()).map_err(|e| e.into())?;
        log_1(&"here 3".into());
        Ok(Self {
            inner: Mutex::new(account),
        })
    }

    #[wasm_bindgen]
    pub async fn sync(&mut self) -> Result<(), DetailledWasmError> {
        self.inner.get_mut().unwrap().sync().await.map_err(|e| e.into())?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn get_balance(&self) -> WasmBalance {
        self.inner.lock().unwrap().get_balance().into()
    }

    #[wasm_bindgen]
    pub fn get_transactions(&self, pagination: WasmPagination) -> Vec<WasmSimpleTransaction> {
        let transaction = self
            .inner
            .lock()
            .unwrap()
            .get_transactions(pagination.into())
            .into_iter()
            .map(|tx| {
                let wasm_tx: WasmSimpleTransaction = tx.into();
                wasm_tx
            })
            .collect::<Vec<_>>();

        transaction
    }
}
