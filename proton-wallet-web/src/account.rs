use std::sync::{Arc, Mutex};

use proton_wallet_common::{
    account::{Account, AccountConfig, SupportedBIPs},
    Address, ExtendedPrivKey, Mnemonic,
};

use wasm_bindgen::prelude::*;

use crate::{
    error::{DetailledWasmError, WasmError},
    types::{
        address::WasmAddress, balance::WasmBalance, defined::WasmNetwork, pagination::WasmPagination,
        transaction::WasmSimpleTransaction, utxo::WasmUtxo,
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
    inner: Arc<Mutex<Account>>,
}

impl WasmAccount {
    pub fn get_inner(&self) -> Arc<Mutex<Account>> {
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
        let mnemonic = Mnemonic::parse(mnemonic_str).map_err(|_| WasmError::InvalidMnemonic.into())?;
        let passphrase = match passphrase {
            Some(passphrase) => passphrase,
            _ => "".to_string(),
        };

        let config: AccountConfig = config.into();
        let mprivkey = ExtendedPrivKey::new_master(config.network.into(), &mnemonic.to_seed(passphrase))
            .map_err(|_| WasmError::InvalidSeed.into())?;

        let account = Account::new(mprivkey, config.into()).map_err(|e| e.into())?;
        Ok(Self {
            inner: Arc::new(Mutex::new(account)),
        })
    }

    #[wasm_bindgen]
    pub async fn sync(&mut self) -> Result<(), DetailledWasmError> {
        self.inner.lock().unwrap().sync().await.map_err(|e| e.into())?;

        Ok(())
    }

    #[wasm_bindgen]
    pub fn owns(&self, address: &WasmAddress) -> bool {
        let address: Address = address.into();

        let mut acc = self.inner.lock().unwrap();
        let wallet = acc.get_mutable_wallet();

        wallet.is_mine(&address.script_pubkey())
    }

    #[wasm_bindgen]
    pub fn get_balance(&self) -> WasmBalance {
        self.inner.lock().unwrap().get_balance().into()
    }

    #[wasm_bindgen]
    pub fn get_utxos(&self) -> Vec<WasmUtxo> {
        let utxos = self
            .inner
            .lock()
            .unwrap()
            .get_utxos()
            .into_iter()
            .map(|utxo| utxo.into())
            .collect::<Vec<WasmUtxo>>();

        utxos
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

#[cfg(test)]
mod tests {
    use crate::types::{address::WasmAddress, defined::WasmNetwork};

    use super::{WasmAccount, WasmAccountConfig, WasmSupportedBIPs};

    // #[actix_rt::test]
    // async fn should_return_true_account_owns_address() {
    //     let mut account = WasmAccount::new("category law logic swear involve banner pink room diesel fragile sunset remove whale lounge captain code hobby lesson material current moment funny vast fade", None, WasmAccountConfig::new(Some(WasmSupportedBIPs::Bip84), Some(WasmNetwork::Testnet), Some(0))).unwrap();
    //     let address = WasmAddress::new("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn".to_string());
    //     account.sync().await.unwrap();

    //     assert!(account.owns(&address))
    // }
}
