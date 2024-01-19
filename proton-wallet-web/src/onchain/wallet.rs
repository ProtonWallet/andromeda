use proton_wallet_common::{
    onchain::{
        account::{build_account_derivation_path, AccountConfig},
        wallet::{Wallet, WalletConfig},
    },
    DerivationPath,
};
use wasm_bindgen::prelude::*;
use web_sys::console::{log_1, log_2};

use super::{
    account::{WasmAccount, WasmScriptType},
    storage::OnchainStorage,
    types::{
        balance::WasmBalance,
        defined::WasmNetwork,
        derivation_path::WasmDerivationPath,
        pagination::WasmPagination,
        transaction::{WasmDetailledTransaction, WasmSimpleTransaction},
        typescript_interfaces::IWasmSimpleTransactionArray,
    },
};
use crate::common::error::{DetailledWasmError, WasmError};

#[wasm_bindgen]
pub struct WasmWallet {
    inner: Wallet<OnchainStorage>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct WasmWalletConfig {
    pub network: WasmNetwork,
    pub no_persist: bool,
}

impl Into<WalletConfig> for &WasmWalletConfig {
    fn into(self) -> WalletConfig {
        WalletConfig {
            network: self.network.into(),
        }
    }
}

#[wasm_bindgen]
impl WasmWalletConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(network: Option<WasmNetwork>, no_persist: Option<bool>) -> Self {
        Self {
            network: network.unwrap_or(WasmNetwork::Bitcoin),
            no_persist: no_persist.unwrap_or(false),
        }
    }
}

#[wasm_bindgen]
impl WasmWallet {
    #[wasm_bindgen(constructor)]
    pub fn new(
        bip39_mnemonic: String,
        bip38_passphrase: Option<String>,
        config: &WasmWalletConfig,
    ) -> Result<WasmWallet, DetailledWasmError> {
        log_2(&"NETWORK 1:".into(), &format!("{:?}", config.network.clone()).into());
        let wallet = Wallet::new(bip39_mnemonic, bip38_passphrase, config.into()).map_err(|e| e.into())?;
        log_2(
            &"NETWORK 1 end:".into(),
            &format!("{:?}", config.network.clone()).into(),
        );
        Ok(Self { inner: wallet })
    }

    #[wasm_bindgen(js_name = addAccount)]
    pub fn add_account(
        &mut self,
        script_type: WasmScriptType,
        account_index: u32,
    ) -> Result<WasmDerivationPath, DetailledWasmError> {
        log_2(&"NETWORK 2:".into(), &format!("{:?}", self.inner.get_network()).into());

        let tmp_derivation_path: DerivationPath = build_account_derivation_path::<()>(AccountConfig::new(
            script_type.into(),
            self.inner.get_network(),
            account_index,
            None,
        ))
        .map_err(|_| WasmError::InvalidDerivationPath.into())?
        .into();

        // In a multi-wallet context, an account must be defined by the BIP32 masterkey (fingerprint), and its derivation path (unique)
        let account_id = format!("{}_{}", self.inner.get_fingerprint(), tmp_derivation_path.to_string());
        let storage = OnchainStorage::new(account_id.clone());

        log_1(&"Before add account".into());
        let derivation_path = self
            .inner
            .add_account(script_type.into(), account_index, storage)
            .map_err(|e| e.into())?;
        log_1(&"After add account".into());

        // assert_eq!(derivation_path, tmp_derivation_path);
        Ok(derivation_path.into())
    }

    #[wasm_bindgen(js_name = getAccount)]
    pub fn get_account(&mut self, account_key: &WasmDerivationPath) -> Option<WasmAccount> {
        let account_key: DerivationPath = account_key.into();
        let account = self.inner.get_account(&account_key);

        account.map(|account| account.into())
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
            .map(|tx| {
                let wasm_tx: WasmSimpleTransaction = tx.into();
                wasm_tx
            })
            .collect::<Vec<_>>();

        Ok(serde_wasm_bindgen::to_value(&transaction).unwrap().into())
    }

    #[wasm_bindgen(js_name = getTransaction)]
    pub async fn get_transaction(
        &self,
        account_key: &WasmDerivationPath,
        txid: String,
    ) -> Result<WasmDetailledTransaction, DetailledWasmError> {
        let account_key: DerivationPath = account_key.into();

        let transaction = self
            .inner
            .get_transaction(&account_key, txid)
            .await
            .map_err(|e| e.into())?;

        Ok(transaction.into())
    }

    #[wasm_bindgen(js_name = getFingerprint)]
    pub fn get_fingerprint(&self) -> String {
        self.inner.get_fingerprint()
    }
}
