use andromeda_bitcoin::account::Account;
use wasm_bindgen::prelude::*;

use super::{
    payment_link::WasmPaymentLink,
    psbt::WasmPsbt,
    storage::{WebOnchainStore, WebOnchainStoreFactory},
    types::{
        address::WasmAddress,
        address_info::WasmAddressInfo,
        balance::WasmBalance,
        derivation_path::WasmDerivationPath,
        pagination::{WasmPagination, WasmSortOrder},
        transaction::{WasmTransactionDetailsArray, WasmTransactionDetailsData},
        utxo::{WasmUtxo, WasmUtxoArray},
    },
    wallet::WasmWallet,
};
use crate::common::{error::ErrorExt, types::WasmScriptType};
#[wasm_bindgen]
pub struct WasmAccount {
    inner: Account<WebOnchainStore>,
}

impl WasmAccount {
    pub fn get_inner(&self) -> Account<WebOnchainStore> {
        self.inner.clone()
    }
}

impl Into<WasmAccount> for &Account<WebOnchainStore> {
    fn into(self) -> WasmAccount {
        WasmAccount { inner: self.clone() }
    }
}

#[wasm_bindgen]
impl WasmAccount {
    #[wasm_bindgen(constructor)]
    pub fn new(
        wallet: &WasmWallet,
        script_type: WasmScriptType,
        derivation_path: WasmDerivationPath,
    ) -> Result<WasmAccount, js_sys::Error> {
        let factory = WebOnchainStoreFactory::new();

        let (mprv, network) = wallet.get_inner().mprv();
        let account = Account::new(mprv, network, script_type.into(), (&derivation_path).into(), factory)
            .map_err(|e| e.to_js_error())?;

        Ok(WasmAccount { inner: account })
    }

    #[wasm_bindgen(js_name = getAddress)]
    pub async fn get_address(&self, index: Option<u32>) -> Result<WasmAddressInfo, js_sys::Error> {
        let account_inner = self.get_inner();

        let address = account_inner
            .get_address(index)
            .await
            .map_err(|e| e.to_js_error())
            .map(|a| a.into())?;

        Ok(address)
    }

    #[wasm_bindgen(js_name = getLastUnusedAddressIndex)]
    pub async fn get_last_unused_address_index(&self) -> Option<u32> {
        let account_inner = self.get_inner();
        let index = account_inner.get_last_unused_address_index().await;
        index
    }

    #[wasm_bindgen(js_name = getBitcoinUri)]
    pub async fn get_bitcoin_uri(
        &mut self,
        index: Option<u32>,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> Result<WasmPaymentLink, js_sys::Error> {
        let mut account_inner = self.get_inner();

        let payment_link: WasmPaymentLink = account_inner
            .get_bitcoin_uri(index, amount, label, message)
            .await
            .map_err(|e| e.to_js_error())?
            .into();

        Ok(payment_link)
    }

    #[wasm_bindgen]
    pub async fn owns(&self, address: &WasmAddress) -> Result<bool, js_sys::Error> {
        let owns = self.inner.owns(&address.into()).await;

        Ok(owns)
    }

    #[wasm_bindgen(js_name = getBalance)]
    pub async fn get_balance(&self) -> Result<WasmBalance, js_sys::Error> {
        let balance: WasmBalance = self.inner.get_balance().await.into();

        Ok(balance)
    }

    #[wasm_bindgen(js_name = getDerivationPath)]
    pub fn get_derivation_path(&self) -> Result<String, js_sys::Error> {
        let derivation_path = self.inner.get_derivation_path().to_string();

        Ok(derivation_path)
    }

    #[wasm_bindgen(js_name = getUtxos)]
    pub async fn get_utxos(&self) -> Result<WasmUtxoArray, js_sys::Error> {
        let utxos = self
            .inner
            .get_utxos()
            .await
            .into_iter()
            .map(|utxo| utxo.into())
            .collect::<Vec<WasmUtxo>>();

        Ok(WasmUtxoArray(utxos))
    }

    #[wasm_bindgen(js_name = getTransactions)]
    pub async fn get_transactions(
        &self,
        pagination: Option<WasmPagination>,
        sort: Option<WasmSortOrder>,
    ) -> Result<WasmTransactionDetailsArray, js_sys::Error> {
        let transactions = self
            .inner
            .get_transactions(pagination.map(|pa| pa.into()), sort.map(|s| s.into()))
            .await
            .map_err(|e| e.to_js_error())?
            .into_iter()
            .map(|tx| WasmTransactionDetailsData { Data: tx.into() })
            .collect::<Vec<_>>();

        Ok(WasmTransactionDetailsArray(transactions))
    }

    #[wasm_bindgen(js_name = getTransaction)]
    pub async fn get_transaction(&self, txid: String) -> Result<WasmTransactionDetailsData, js_sys::Error> {
        let transaction = self.inner.get_transaction(txid).await.map_err(|e| e.to_js_error())?;

        Ok(WasmTransactionDetailsData {
            Data: transaction.into(),
        })
    }

    #[wasm_bindgen(js_name = hasSyncData)]
    pub async fn has_sync_data(&self) -> bool {
        self.inner.has_sync_data().await
    }

    #[wasm_bindgen(js_name = insertUnconfirmedTransaction)]
    pub async fn insert_unconfirmed_tx(&self, psbt: &WasmPsbt) -> Result<(), js_sys::Error> {
        let transaction = psbt.get_inner().extract_tx().map_err(|e| e.to_js_error())?;

        self.inner
            .insert_unconfirmed_tx(transaction)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(())
    }
}
