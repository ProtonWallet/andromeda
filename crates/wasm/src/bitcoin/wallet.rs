use std::str::FromStr;

use andromeda_bitcoin::{wallet::Wallet, BdkMemoryDatabase, DerivationPath};
use andromeda_common::{error::Error, ScriptType};
use wasm_bindgen::prelude::*;

use super::{
    account::WasmAccount,
    types::{
        balance::WasmBalance,
        derivation_path::WasmDerivationPath,
        pagination::WasmPagination,
        transaction::{WasmSimpleTransaction, WasmTransactionDetails},
        typescript_interfaces::IWasmSimpleTransactionArray,
    },
};
use crate::common::{
    error::{DetailledWasmError, WasmError},
    types::WasmNetwork,
};

#[wasm_bindgen]
pub struct WasmWallet {
    inner: Wallet<BdkMemoryDatabase>,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "[u8, String]")]
    pub type AccountConfigTupple;
}

#[wasm_bindgen]
impl WasmWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(
        network: WasmNetwork,
        bip39_mnemonic: String,
        bip38_passphrase: Option<String>,
        accounts: Option<Vec<AccountConfigTupple>>,
    ) -> Result<WasmWallet, DetailledWasmError> {
        let accounts = accounts.map_or(Vec::new(), |accounts| {
            accounts
                .into_iter()
                .map(|acc| -> Result<(ScriptType, DerivationPath, BdkMemoryDatabase), _> {
                    let (script_type, derivation_path): (u8, String) =
                        serde_wasm_bindgen::from_value(acc.into()).map_err(|_| WasmError::InvalidData)?;

                    let derivation_path =
                        DerivationPath::from_str(&derivation_path).map_err(|_| WasmError::InvalidDerivationPath)?;
                    let script_type = script_type.try_into().map_err(|_| WasmError::InvalidScriptType)?;

                    let config = (script_type, derivation_path, BdkMemoryDatabase::new());

                    Ok::<(ScriptType, DerivationPath, BdkMemoryDatabase), WasmError>(config)
                })
                .filter_map(Result::ok)
                .collect::<Vec<_>>()
        });

        let wallet = Wallet::new_with_accounts(network.into(), bip39_mnemonic, bip38_passphrase, accounts)
            .map_err(|e| e.into())?;

        Ok(Self { inner: wallet })
    }

    #[wasm_bindgen(js_name = addAccount)]
    pub fn add_account(
        &mut self,
        script_type: u8,
        derivation_path: String,
    ) -> Result<WasmDerivationPath, DetailledWasmError> {
        // In a multi-wallet context, an account must be defined by the BIP32 masterkey
        // (fingerprint), and its derivation path (unique)
        let storage = BdkMemoryDatabase::new();

        let derivation_path =
            DerivationPath::from_str(&derivation_path).map_err(|_| WasmError::InvalidDerivationPath.into())?;

        let script_type = script_type.try_into().map_err(|e: Error| e.into())?;

        self.inner
            .add_account(script_type, derivation_path.clone(), storage)
            .map_err(|e| e.into())?;

        Ok(derivation_path.into())
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
    pub async fn get_balance(&self) -> Result<WasmBalance, DetailledWasmError> {
        let balance = self.inner.get_balance().await.map_err(|e| e.into())?;
        Ok(balance.into())
    }

    #[wasm_bindgen(js_name = getTransactions)]
    pub async fn get_transactions(
        &self,
        pagination: Option<WasmPagination>,
    ) -> Result<IWasmSimpleTransactionArray, DetailledWasmError> {
        let transaction = self
            .inner
            .get_transactions(pagination.map(|pa| pa.into()), true)
            .await
            .map_err(|e| e.into())?
            .into_iter()
            .map(|tx| tx.into())
            .collect::<Vec<_>>();

        Ok(serde_wasm_bindgen::to_value(&transaction).unwrap().into())
    }

    #[wasm_bindgen(js_name = getTransaction)]
    pub async fn get_transaction(
        &self,
        account_key: &WasmDerivationPath,
        txid: String,
    ) -> Result<WasmTransactionDetailsData, DetailledWasmError> {
        let account_key: DerivationPath = account_key.into();

        let transaction = self
            .inner
            .get_transaction(&account_key, txid)
            .await
            .map_err(|e| e.into())?;

        Ok(WasmTransactionDetailsData {
            Data: transaction.into(),
        })
    }

    #[wasm_bindgen(js_name = getFingerprint)]
    pub fn get_fingerprint(&self) -> String {
        self.inner.get_fingerprint()
    }
}
