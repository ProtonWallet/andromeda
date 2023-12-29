use std::{fmt::Display, sync::Arc};

use proton_wallet_common::{
    account::{Account, AccountConfig, SupportedBIPs},
    async_rw_lock::AsyncRwLock,
    Address,
};

use wasm_bindgen::prelude::*;

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
        typescript_interfaces::{IWasmSimpleTransactionArray, IWasmUtxoArray},
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
    inner: Arc<AsyncRwLock<Account<OnchainStorage>>>,
}

impl WasmAccount {
    pub fn get_inner(&self) -> Arc<AsyncRwLock<Account<OnchainStorage>>> {
        self.inner.clone()
    }
}

impl Into<WasmAccount> for &Arc<AsyncRwLock<Account<OnchainStorage>>> {
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

pub struct WasmTransactionTime {
    pub confirmed: bool,
    pub confirmation_time: Option<u64>,
    pub last_seen: Option<u64>,
}

#[wasm_bindgen]
impl WasmAccount {
    #[wasm_bindgen(js_name = hasSyncData)]
    pub async fn has_sync_data(&self) -> bool {
        match self.get_inner().read().await {
            Ok(inner) => inner.get_storage().exists(),
            _ => false,
        }
    }

    #[wasm_bindgen(js_name = getBitcoinUri)]
    pub async fn get_bitcoin_uri(
        &mut self,
        index: Option<u32>,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> Result<WasmPaymentLink, DetailledWasmError> {
        let account_inner = self.get_inner();

        let payment_link: WasmPaymentLink = account_inner
            .write()
            .await
            .map_err(|_| WasmError::LockError.into())?
            .get_bitcoin_uri(index, amount, label, message)
            .into();

        account_inner.release_write_lock();
        Ok(payment_link)
    }

    #[wasm_bindgen]
    pub async fn owns(&self, address: &WasmAddress) -> Result<bool, DetailledWasmError> {
        let address: Address = address.into();

        let account = self.inner.read().await.map_err(|_| WasmError::LockError.into())?;
        let wallet = account.get_wallet();

        Ok(wallet.is_mine(&address.script_pubkey()))
    }

    #[wasm_bindgen(js_name = getBalance)]
    pub async fn get_balance(&self) -> Result<WasmBalance, DetailledWasmError> {
        let balance: WasmBalance = self
            .inner
            .read()
            .await
            .map_err(|_| WasmError::LockError.into())?
            .get_balance()
            .into();

        Ok(balance)
    }

    #[wasm_bindgen(js_name = getDerivationPath)]
    pub async fn get_derivation_path(&self) -> Result<String, DetailledWasmError> {
        let derivation_path = self
            .inner
            .read()
            .await
            .map_err(|_| WasmError::LockError.into())?
            .get_derivation_path()
            .to_string();

        Ok(derivation_path)
    }

    #[wasm_bindgen(js_name = getUtxos)]
    pub async fn get_utxos(&self) -> Result<IWasmUtxoArray, DetailledWasmError> {
        let utxos = self
            .inner
            .read()
            .await
            .map_err(|_| WasmError::LockError.into())?
            .get_utxos()
            .into_iter()
            .map(|utxo| utxo.into())
            .collect::<Vec<WasmUtxo>>();

        Ok(serde_wasm_bindgen::to_value(&utxos).unwrap().into())
    }

    #[wasm_bindgen(js_name = getTransactions)]
    pub async fn get_transactions(
        &self,
        pagination: Option<WasmPagination>,
    ) -> Result<IWasmSimpleTransactionArray, DetailledWasmError> {
        let transactions = self
            .inner
            .read()
            .await
            .map_err(|_| WasmError::LockError.into())?
            .get_transactions(pagination.map(|pa| pa.into()), true)
            .into_iter()
            .map(|tx| {
                let wasm_tx: WasmSimpleTransaction = tx.into();
                wasm_tx
            })
            .collect::<Vec<_>>();

        Ok(serde_wasm_bindgen::to_value(&transactions).unwrap().into())
    }

    #[wasm_bindgen(js_name = getTransaction)]
    pub async fn get_transaction(&self, txid: String) -> Result<WasmDetailledTransaction, DetailledWasmError> {
        let transaction = self
            .inner
            .read()
            .await
            .map_err(|_| WasmError::LockError.into())?
            .get_transaction(txid)
            .map_err(|e| e.into())?;

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
