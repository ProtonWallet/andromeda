use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use andromeda_bitcoin::{
    account::{Account, AccountConfig, ScriptType},
    BdkMemoryDatabase,
};

use wasm_bindgen::prelude::*;

use super::{
    payment_link::WasmPaymentLink,
    types::{
        address::WasmAddress,
        balance::WasmBalance,
        defined::WasmNetwork,
        pagination::WasmPagination,
        transaction::{WasmSimpleTransaction, WasmTransactionDetails},
        typescript_interfaces::{IWasmSimpleTransactionArray, IWasmUtxoArray},
        utxo::WasmUtxo,
    },
};
use crate::common::error::{DetailledWasmError, WasmError};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmScriptType {
    Legacy,
    NestedSegwit,
    NativeSegwit,
    Taproot,
}

impl Display for WasmScriptType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WasmScriptType::Legacy => "Legacy",
                WasmScriptType::NestedSegwit => "NestedSegwit",
                WasmScriptType::NativeSegwit => "NativeSegwit",
                WasmScriptType::Taproot => "Taproot",
            }
        )
    }
}

impl Into<ScriptType> for WasmScriptType {
    fn into(self) -> ScriptType {
        match self {
            WasmScriptType::Legacy => ScriptType::Legacy,
            WasmScriptType::NestedSegwit => ScriptType::NestedSegwit,
            WasmScriptType::NativeSegwit => ScriptType::NativeSegwit,
            WasmScriptType::Taproot => ScriptType::Taproot,
        }
    }
}

#[wasm_bindgen]
pub struct WasmAccount {
    inner: Arc<RwLock<Account<BdkMemoryDatabase>>>,
}

impl WasmAccount {
    pub fn get_inner(&self) -> Arc<RwLock<Account<BdkMemoryDatabase>>> {
        self.inner.clone()
    }
}

impl Into<WasmAccount> for &Arc<RwLock<Account<BdkMemoryDatabase>>> {
    fn into(self) -> WasmAccount {
        WasmAccount { inner: self.clone() }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmAccountConfig {
    pub script_type: WasmScriptType,
    pub network: WasmNetwork,
    pub account_index: u32,
}

impl Into<AccountConfig> for WasmAccountConfig {
    fn into(self) -> AccountConfig {
        AccountConfig {
            script_type: self.script_type.into(),
            account_index: self.account_index,
            network: self.network.into(),
            multisig_threshold: None,
        }
    }
}

#[wasm_bindgen]
impl WasmAccountConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(script_type: WasmScriptType, network: Option<WasmNetwork>, account_index: Option<u32>) -> Self {
        Self {
            script_type: script_type.into(),
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
            .expect("lock")
            .get_bitcoin_uri(index, amount, label, message)
            .map_err(|e| e.into())?
            .into();

        Ok(payment_link)
    }

    #[wasm_bindgen]
    pub async fn owns(&self, address: &WasmAddress) -> Result<bool, DetailledWasmError> {
        let owns = self
            .inner
            .read()
            .expect("lock")
            .owns(&address.into())
            .map_err(|e| e.into())?;

        Ok(owns)
    }

    #[wasm_bindgen(js_name = getBalance)]
    pub async fn get_balance(&self) -> Result<WasmBalance, DetailledWasmError> {
        let balance: WasmBalance = self
            .inner
            .read()
            .expect("lock")
            .get_balance()
            .map_err(|e| e.into())?
            .into();

        Ok(balance)
    }

    #[wasm_bindgen(js_name = getDerivationPath)]
    pub async fn get_derivation_path(&self) -> Result<String, DetailledWasmError> {
        let derivation_path = self.inner.read().expect("lock").get_derivation_path().to_string();

        Ok(derivation_path)
    }

    #[wasm_bindgen(js_name = getUtxos)]
    pub async fn get_utxos(&self) -> Result<IWasmUtxoArray, DetailledWasmError> {
        let utxos = self
            .inner
            .read()
            .expect("lock")
            .get_utxos()
            .map_err(|e| e.into())?
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
            .expect("lock")
            .get_transactions(pagination.map(|pa| pa.into()), true)
            .map_err(|e| e.into())?
            .into_iter()
            .map(|tx| {
                let wasm_tx: WasmSimpleTransaction = tx.into();
                wasm_tx
            })
            .collect::<Vec<_>>();

        Ok(serde_wasm_bindgen::to_value(&transactions).unwrap().into())
    }

    #[wasm_bindgen(js_name = getTransaction)]
    pub async fn get_transaction(&self, txid: String) -> Result<WasmTransactionDetails, DetailledWasmError> {
        let transaction = self
            .inner
            .read()
            .expect("lock")
            .get_transaction(txid)
            .map_err(|e| e.into())?;

        Ok(transaction.into())
    }
}
