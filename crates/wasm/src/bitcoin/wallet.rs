use std::str::FromStr;

use andromeda_bitcoin::{error::Error as BitcoinError, wallet::Wallet, DerivationPath};
use andromeda_common::error::Error;
use wasm_bindgen::prelude::*;

use super::{
    account::WasmAccount,
    storage::WalletWebPersisterFactory,
    types::{
        balance::WasmBalanceWrapper,
        derivation_path::WasmDerivationPath,
        pagination::{WasmPagination, WasmSortOrder},
        transaction::{WasmTransactionDetailsArray, WasmTransactionDetailsData},
    },
};
use crate::{
    api::WasmProtonWalletApiClient,
    common::{
        error::ErrorExt,
        types::{WasmNetwork, WasmScriptType},
    },
};

#[wasm_bindgen]
pub struct WasmWallet {
    inner: Wallet,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "[u8, String]")]
    pub type AccountConfigTupple;
}

impl WasmWallet {
    pub fn get_inner(&self) -> &Wallet {
        &self.inner
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmDiscoveredAccount(pub WasmScriptType, pub u32, pub WasmDerivationPath);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmDiscoveredAccounts {
    pub data: Vec<WasmDiscoveredAccount>,
}

#[wasm_bindgen]
impl WasmWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(
        network: WasmNetwork,
        bip39_mnemonic: String,
        bip38_passphrase: Option<String>,
    ) -> Result<WasmWallet, js_sys::Error> {
        let wallet = Wallet::new(network.into(), bip39_mnemonic, bip38_passphrase).map_err(|e| e.to_js_error())?;

        Ok(Self { inner: wallet })
    }

    #[wasm_bindgen(js_name = addAccount)]
    pub fn add_account(&mut self, script_type: u8, derivation_path: String) -> Result<WasmAccount, js_sys::Error> {
        let factory = WalletWebPersisterFactory;

        // In a multi-wallet context, an account must be defined by the BIP32 masterkey
        // (fingerprint), and its derivation path (unique)
        let derivation_path =
            DerivationPath::from_str(&derivation_path).map_err(|e| BitcoinError::from(e).to_js_error())?;

        let script_type = script_type.try_into().map_err(|e: Error| e.to_js_error())?;

        let account_arc = self
            .inner
            .add_account(script_type, derivation_path.clone(), factory)
            .map_err(|e| BitcoinError::from(e).to_js_error())?;

        Ok(account_arc.into())
    }

    #[wasm_bindgen(js_name = discoverAccounts)]
    pub async fn discover_accounts(
        &self,
        api_client: &WasmProtonWalletApiClient,
    ) -> Result<WasmDiscoveredAccounts, js_sys::Error> {
        let factory = WalletWebPersisterFactory;

        let accounts = self
            .inner
            .discover_accounts(api_client.into(), factory, None, None)
            .await
            .map_err(|e| BitcoinError::from(e).to_js_error())?;

        let discovered_accounts = accounts
            .into_iter()
            .map(|(s, i, d)| WasmDiscoveredAccount(s.into(), i, d.into()))
            .collect::<Vec<_>>();

        Ok(WasmDiscoveredAccounts {
            data: discovered_accounts,
        })
    }

    #[wasm_bindgen(js_name = getAccount)]
    pub fn get_account(&mut self, derivation_path: String) -> Option<WasmAccount> {
        let derivation_path = DerivationPath::from_str(&derivation_path).ok();

        if derivation_path.is_none() {
            return None;
        }

        self.inner
            .get_account(&derivation_path.unwrap())
            .map(|account| account.into())
    }

    #[wasm_bindgen(js_name = getBalance)]
    pub async fn get_balance(&self) -> Result<WasmBalanceWrapper, js_sys::Error> {
        let balance = self.inner.get_balance().await.map_err(|e| e.to_js_error())?;

        Ok(WasmBalanceWrapper { data: balance.into() })
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
    pub async fn get_transaction(
        &self,
        account_key: &WasmDerivationPath,
        txid: String,
    ) -> Result<WasmTransactionDetailsData, js_sys::Error> {
        let account_key: DerivationPath = account_key.into();

        let transaction = self
            .inner
            .get_transaction(&account_key, txid)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmTransactionDetailsData {
            Data: transaction.into(),
        })
    }

    #[wasm_bindgen(js_name = getFingerprint)]
    pub fn get_fingerprint(&self) -> String {
        self.inner.get_fingerprint()
    }

    #[wasm_bindgen(js_name = clearStore)]
    pub async fn clear_store(&self) -> Result<(), js_sys::Error> {
        self.inner.clear_store().await.map_err(|e| e.to_js_error())?;
        Ok(())
    }
}
