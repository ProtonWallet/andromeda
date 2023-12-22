use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use proton_wallet_common::{
    account::{Account, AccountConfig, SupportedBIPs},
    Address,
};

use wasm_bindgen::prelude::*;
use web_sys::console::log_2;

use crate::{
    error::{DetailledWasmError, WasmError},
    payment_link::WasmPaymentLink,
    storage::OnchainStorage,
    types::{
        address::WasmAddress,
        balance::WasmBalance,
        defined::WasmNetwork,
        pagination::WasmPagination,
        transaction::{WasmDetailledTransaction, WasmSimpleTransaction},
        utxo::WasmUtxo,
    },
};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmSupportedBIPs {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl Display for WasmSupportedBIPs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WasmSupportedBIPs::Bip44 => "44",
                WasmSupportedBIPs::Bip49 => "49",
                WasmSupportedBIPs::Bip84 => "84",
                WasmSupportedBIPs::Bip86 => "86",
            }
        )
    }
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
    inner: Arc<RwLock<Account<OnchainStorage>>>,
}

impl WasmAccount {
    pub fn get_inner(&self) -> Arc<RwLock<Account<OnchainStorage>>> {
        self.inner.clone()
    }
}

impl Into<WasmAccount> for &Arc<RwLock<Account<OnchainStorage>>> {
    fn into(self) -> WasmAccount {
        WasmAccount { inner: self.clone() }
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
    #[wasm_bindgen]
    pub fn has_sync_data(&self) -> bool {
        match self.get_inner().read() {
            Ok(inner) => inner.get_storage().exists(),
            _ => false,
        }
    }

    #[wasm_bindgen]
    pub fn get_bitcoin_uri(
        &mut self,
        index: Option<u32>,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> Result<WasmPaymentLink, DetailledWasmError> {
        let payment_link: WasmPaymentLink = self
            .get_inner()
            .write()
            .map_err(|_| WasmError::LockError.into())?
            .get_bitcoin_uri(index, amount, label, message)
            .into();

        Ok(payment_link)
    }

    #[wasm_bindgen]
    pub fn owns(&self, address: &WasmAddress) -> Result<bool, DetailledWasmError> {
        let address: Address = address.into();

        let acc = self.inner.read().map_err(|_| WasmError::LockError.into())?;
        let wallet = acc.get_wallet();

        Ok(wallet.is_mine(&address.script_pubkey()))
    }

    #[wasm_bindgen]
    pub fn get_balance(&self) -> Result<WasmBalance, DetailledWasmError> {
        let balance: WasmBalance = self
            .inner
            .read()
            .map_err(|_| WasmError::LockError.into())?
            .get_balance()
            .into();

        Ok(balance)
    }

    #[wasm_bindgen]
    pub fn get_derivation_path(&self) -> Result<String, DetailledWasmError> {
        let derivation_path = self
            .inner
            .read()
            .map_err(|_| WasmError::LockError.into())?
            .get_derivation_path()
            .to_string();

        Ok(derivation_path)
    }

    #[wasm_bindgen]
    pub fn get_utxos(&self) -> Result<Vec<WasmUtxo>, DetailledWasmError> {
        let utxos = self
            .inner
            .read()
            .map_err(|_| WasmError::LockError.into())?
            .get_utxos()
            .into_iter()
            .map(|utxo| utxo.into())
            .collect::<Vec<WasmUtxo>>();

        Ok(utxos)
    }

    #[wasm_bindgen]
    pub fn get_transactions(
        &self,
        pagination: Option<WasmPagination>,
    ) -> Result<Vec<WasmSimpleTransaction>, DetailledWasmError> {
        log_2(
            &"Start get_transactions".into(),
            &self.get_derivation_path().unwrap().to_string().into(),
        );

        let transaction = self
            .inner
            .read()
            .map_err(|_| WasmError::LockError.into())?
            .get_transactions(
                match pagination {
                    Some(pagination) => Some(pagination.into()),
                    _ => None,
                },
                true,
            )
            .into_iter()
            .map(|tx| {
                let wasm_tx: WasmSimpleTransaction = tx.into();
                wasm_tx
            })
            .collect::<Vec<_>>();

        log_2(
            &"Finished get_transactions".into(),
            &self.get_derivation_path().unwrap().to_string().into(),
        );
        Ok(transaction)
    }

    #[wasm_bindgen]
    pub fn get_transaction(&self, txid: String) -> Result<WasmDetailledTransaction, DetailledWasmError> {
        log_2(
            &"Start SINGLE get_transaction".into(),
            &self.get_derivation_path().unwrap().to_string().into(),
        );

        let transaction = self
            .inner
            .read()
            .map_err(|_| WasmError::LockError.into())?
            .get_transaction(txid)
            .map_err(|e| e.into())?;

        log_2(
            &"Finish SINGLE get_transaction".into(),
            &self.get_derivation_path().unwrap().to_string().into(),
        );

        Ok(transaction.into())
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
