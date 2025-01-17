use std::sync::Arc;

use andromeda_bitcoin::account::Account;
use wasm_bindgen::prelude::*;

use super::{
    blockchain_client::WasmBlockchainClient,
    psbt::WasmPsbt,
    storage::{WalletWebConnector, WalletWebPersister, WalletWebPersisterFactory},
    types::{
        address::{WasmAddress, WasmAddressDetailsArray, WasmAddressDetailsData},
        address_info::WasmAddressInfo,
        balance::{WasmBalance, WasmBalanceWrapper},
        derivation_path::WasmDerivationPath,
        pagination::{WasmPagination, WasmSortOrder},
        transaction::{WasmTransactionDetailsArray, WasmTransactionDetailsData},
        utxo::{WasmUtxo, WasmUtxoArray},
    },
    wallet::WasmWallet,
};
use crate::common::{
    error::ErrorExt,
    types::{WasmKeychainKind, WasmNetwork, WasmScriptType},
};

#[wasm_bindgen]
pub struct WasmAccount {
    inner: Arc<Account<WalletWebConnector, WalletWebPersister>>,
}

impl WasmAccount {
    pub fn get_inner(&self) -> Arc<Account<WalletWebConnector, WalletWebPersister>> {
        self.inner.clone()
    }
}

impl Into<WasmAccount> for Arc<Account<WalletWebConnector, WalletWebPersister>> {
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
        let factory = WalletWebPersisterFactory;

        let (mprv, network) = wallet.get_inner().mprv();
        let account = Account::new(mprv, network, script_type.into(), (&derivation_path).into(), factory)
            .map_err(|e| e.to_js_error())?;

        Ok(Arc::new(account).into())
    }

    #[wasm_bindgen(js_name = markReceiveAddressesUsedTo)]
    pub async fn mark_receive_addresses_used_to(&mut self, from: u32, to: Option<u32>) -> Result<(), js_sys::Error> {
        let account_inner = self.get_inner();

        account_inner
            .mark_receive_addresses_used_to(from, to)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = getNextReceiveAddress)]
    pub async fn get_next_receive_address(&self) -> Result<WasmAddressInfo, js_sys::Error> {
        let account_inner = self.get_inner();

        let address = account_inner
            .get_next_receive_address()
            .await
            .map_err(|e| e.to_js_error())
            .map(|a| a.into())?;

        Ok(address)
    }

    #[wasm_bindgen(js_name = peekReceiveAddress)]
    pub async fn peek_receive_address(&self, index: u32) -> Result<WasmAddressInfo, js_sys::Error> {
        let account_inner = self.get_inner();

        let address = account_inner
            .peek_receive_address(index)
            .await
            .map_err(|e| e.to_js_error())
            .map(|a| a.into())?;

        Ok(address)
    }

    #[wasm_bindgen]
    pub async fn owns(&self, address: &WasmAddress) -> Result<bool, js_sys::Error> {
        let owns = self.inner.owns(&address.into()).await;

        Ok(owns)
    }

    #[wasm_bindgen(js_name = getBalance)]
    pub async fn get_balance(&self) -> Result<WasmBalanceWrapper, js_sys::Error> {
        let balance: WasmBalance = self.inner.get_balance().await.into();

        Ok(WasmBalanceWrapper { data: balance })
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

    #[wasm_bindgen(js_name = getAddress)]
    pub async fn get_address(
        &self,
        network: WasmNetwork,
        address_str: String,
        client: &WasmBlockchainClient,
        force_sync: Option<bool>,
    ) -> Result<Option<WasmAddressDetailsData>, js_sys::Error> {
        let address_details = self
            .inner
            .get_address(network.into(), address_str, client.into(), force_sync.unwrap_or(false))
            .await
            .map_err(|e| e.to_js_error())?
            .map(|address| WasmAddressDetailsData { Data: address.into() });

        Ok(address_details)
    }

    #[wasm_bindgen(js_name = getAddresses)]
    pub async fn get_addresses(
        &self,
        pagination: WasmPagination,
        client: &WasmBlockchainClient,
        keychain: WasmKeychainKind,
        force_sync: Option<bool>,
    ) -> Result<WasmAddressDetailsArray, js_sys::Error> {
        let address_details = self
            .inner
            .get_addresses(
                pagination.into(),
                client.into(),
                keychain.into(),
                force_sync.unwrap_or(false),
            )
            .await
            .map_err(|e| e.to_js_error())?
            .into_iter()
            .map(|address| WasmAddressDetailsData { Data: address.into() })
            .collect::<Vec<_>>();

        Ok(WasmAddressDetailsArray(address_details))
    }

    #[wasm_bindgen(js_name = getHighestUsedAddressIndexInOutput)]
    pub async fn get_highest_used_address_index_in_output(
        &self,
        keychain: WasmKeychainKind,
    ) -> Result<Option<u32>, js_sys::Error> {
        Ok(self
            .inner
            .get_highest_used_address_index_in_output(keychain.into())
            .await
            .map_err(|e| e.to_js_error())?)
    }

    #[wasm_bindgen(js_name = getTransactions)]
    pub async fn get_transactions(
        &self,
        pagination: WasmPagination,
        sort: Option<WasmSortOrder>,
    ) -> Result<WasmTransactionDetailsArray, js_sys::Error> {
        let transactions = self
            .inner
            .get_transactions(pagination.into(), sort.map(|s| s.into()))
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

    #[wasm_bindgen(js_name = bumpTransactionsFees)]
    pub async fn bump_transactions_fees(
        &self,
        network: WasmNetwork,
        txid: String,
        fees: u64,
    ) -> Result<WasmPsbt, js_sys::Error> {
        let psbt = self
            .inner
            .bump_transactions_fees(txid, fees)
            .await
            .map_err(|e| e.to_js_error())?;

        let wasm_psbt = WasmPsbt::from_psbt(&psbt, network.into())?;

        Ok(wasm_psbt)
    }

    #[wasm_bindgen(js_name = clearStore)]
    pub async fn clear_store(&self) -> Result<(), js_sys::Error> {
        self.inner.clear_store().map_err(|e| e.to_js_error())?;
        Ok(())
    }
}
