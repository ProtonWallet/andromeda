use andromeda_api::wallet::{
    Account, CreateWalletAccountRequestBody, CreateWalletRequestBody, CreateWalletTransactionRequestBody, WalletClient,
    WalletData, WalletTransaction,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::{bitcoin::types::derivation_path::WasmDerivationPath, common::error::WasmError};

#[wasm_bindgen]
pub struct WasmWalletClient(WalletClient);

impl From<WalletClient> for WasmWalletClient {
    fn from(value: WalletClient) -> Self {
        Self(value)
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiWallet {
    pub ID: String,
    pub Name: String,
    pub IsImported: u8,
    pub Priority: u8,
    pub Type: u8,
    pub HasPassphrase: u8,
    pub Status: u8,
    pub Mnemonic: Option<String>,
    pub Fingerprint: Option<String>,
    pub PublicKey: Option<String>,
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmWalletKey {
    pub UserKeyID: String,
    pub WalletKey: String,
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmWalletSettings {
    pub HideAccounts: u8,
    pub InvoiceDefaultDescription: Option<String>,
    pub InvoiceExpirationTime: u64,
    pub MaxChannelOpeningFee: u64,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmWalletData {
    pub Wallet: WasmApiWallet,
    pub WalletKey: WasmWalletKey,
    pub WalletSettings: WasmWalletSettings,
}

impl From<WalletData> for WasmWalletData {
    fn from(value: WalletData) -> Self {
        WasmWalletData {
            Wallet: WasmApiWallet {
                ID: value.Wallet.ID,
                Name: value.Wallet.Name,
                IsImported: value.Wallet.IsImported,
                Priority: value.Wallet.Priority,
                Type: value.Wallet.Type,
                HasPassphrase: value.Wallet.HasPassphrase,
                Status: value.Wallet.Status,
                Mnemonic: value.Wallet.Mnemonic,
                PublicKey: value.Wallet.PublicKey,
                Fingerprint: value.Wallet.Fingerprint,
            },
            WalletKey: WasmWalletKey {
                UserKeyID: value.WalletKey.UserKeyID,
                WalletKey: value.WalletKey.WalletKey,
            },
            WalletSettings: WasmWalletSettings {
                HideAccounts: value.WalletSettings.HideAccounts,
                InvoiceDefaultDescription: value.WalletSettings.InvoiceDefaultDescription,
                InvoiceExpirationTime: value.WalletSettings.InvoiceExpirationTime,
                MaxChannelOpeningFee: value.WalletSettings.MaxChannelOpeningFee,
            },
        }
    }
}

#[wasm_bindgen]
impl WasmWalletData {
    #[wasm_bindgen]
    pub fn from_parts(wallet: WasmApiWallet, key: WasmWalletKey, settings: WasmWalletSettings) -> Self {
        Self {
            Wallet: wallet,
            WalletKey: key,
            WalletSettings: settings,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmWalletAccount {
    pub ID: String,
    pub DerivationPath: String,
    pub Label: String,
    pub ScriptType: u8,
}

// We need this wrapper because unfortunately, tsify doesn't support VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmWalletAccountData {
    pub Account: WasmWalletAccount,
}

impl From<Account> for WasmWalletAccount {
    fn from(value: Account) -> Self {
        WasmWalletAccount {
            ID: value.ID,
            DerivationPath: value.DerivationPath,
            Label: value.Label,
            ScriptType: value.ScriptType,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmWalletTransaction {
    pub ID: String,
    pub WalletID: String,
    pub Label: String,
    pub TransactionID: String,
}

impl From<WalletTransaction> for WasmWalletTransaction {
    fn from(value: WalletTransaction) -> Self {
        WasmWalletTransaction {
            ID: value.ID,
            WalletID: value.WalletID,
            Label: value.Label,
            TransactionID: value.TransactionID,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmWalletDataArray(pub Vec<WasmWalletData>);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmWalletAccountArray(pub Vec<WasmWalletAccountData>);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmWalletTransactionArray(pub Vec<WasmWalletTransaction>);

#[wasm_bindgen]
impl WasmWalletClient {
    #[wasm_bindgen(js_name = "getWallets")]
    pub async fn get_wallets(&self) -> Result<WasmWalletDataArray, WasmError> {
        let wallets = self
            .0
            .get_wallets()
            .await
            .map_err(|e| e.into())
            .map(|wallets| wallets.into_iter().map(|wallet| wallet.into()).collect::<Vec<_>>())?;

        Ok(WasmWalletDataArray(wallets))
    }

    #[wasm_bindgen(js_name = "createWallet")]
    pub async fn create_wallet(
        &self,
        name: String,
        is_imported: bool,
        wallet_type: u8,
        has_passphrase: bool,
        user_key_id: String,
        wallet_key: String,
        mnemonic: Option<String>,
        fingerprint: Option<String>,
        public_key: Option<String>,
    ) -> Result<WasmWalletData, WasmError> {
        let payload = CreateWalletRequestBody {
            Name: name,
            IsImported: match is_imported {
                true => 1,
                false => 0,
            },
            Type: wallet_type,
            HasPassphrase: match has_passphrase {
                true => 1,
                false => 0,
            },
            UserKeyId: user_key_id,
            WalletKey: wallet_key,
            Mnemonic: mnemonic,
            Fingerprint: fingerprint,
            PublicKey: public_key,
        };

        self.0
            .create_wallet(payload)
            .await
            .map_err(|e| e.into())
            .map(|wallet| wallet.into())
    }

    #[wasm_bindgen(js_name = "getWalletAccounts")]
    pub async fn get_wallet_accounts(&self, wallet_id: String) -> Result<WasmWalletAccountArray, WasmError> {
        let wallet_accounts = self
            .0
            .get_wallet_accounts(wallet_id)
            .await
            .map_err(|e| e.into())
            .map(|accounts| {
                accounts
                    .into_iter()
                    .map(|account| WasmWalletAccountData {
                        Account: account.into(),
                    })
                    .collect::<Vec<_>>()
            })?;

        Ok(WasmWalletAccountArray(wallet_accounts))
    }

    #[wasm_bindgen(js_name = "createWalletAccount")]
    pub async fn create_wallet_account(
        &self,
        wallet_id: String,
        derivation_path: WasmDerivationPath,
        label: String,
        script_type: u8,
    ) -> Result<WasmWalletAccountData, WasmError> {
        let payload = CreateWalletAccountRequestBody {
            DerivationPath: derivation_path.inner().to_string(),
            Label: label,
            ScriptType: script_type,
        };

        let account = self
            .0
            .create_wallet_account(wallet_id, payload)
            .await
            .map_err(|e| e.into())
            .map(|account| account.into())?;

        Ok(WasmWalletAccountData { Account: account })
    }

    #[wasm_bindgen(js_name = "updateWalletAccountLabel")]
    pub async fn update_wallet_account_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        label: String,
    ) -> Result<WasmWalletAccountData, WasmError> {
        let account = self
            .0
            .update_wallet_account_label(wallet_id, wallet_account_id, label)
            .await
            .map_err(|e| e.into())
            .map(|account| account.into())?;

        Ok(WasmWalletAccountData { Account: account })
    }

    #[wasm_bindgen(js_name = "deleteWalletAccount")]
    pub async fn delete_wallet_account(&self, wallet_id: String, wallet_account_id: String) -> Result<(), WasmError> {
        self.0
            .delete_wallet_account(wallet_id, wallet_account_id)
            .await
            .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name = "getWalletTransactions")]
    pub async fn get_wallet_transactions(&self, wallet_id: String) -> Result<WasmWalletTransactionArray, WasmError> {
        let wallet_transactions = self
            .0
            .get_wallet_transactions(wallet_id)
            .await
            .map_err(|e| e.into())
            .map(|transactions| {
                transactions
                    .into_iter()
                    .map(|transaction| transaction.into())
                    .collect::<Vec<_>>()
            })?;

        Ok(WasmWalletTransactionArray(wallet_transactions))
    }

    #[wasm_bindgen(js_name = "createWalletTransaction")]
    pub async fn create_wallet_transaction(
        &self,
        wallet_id: String,
        label: String,
        txid: String,
    ) -> Result<WasmWalletTransaction, WasmError> {
        let payload = CreateWalletTransactionRequestBody {
            Label: label,
            TransactionID: txid,
        };

        self.0
            .create_wallet_transaction(wallet_id, payload)
            .await
            .map_err(|e| e.into())
            .map(|t| t.into())
    }

    #[wasm_bindgen(js_name = "updateWalletTransactionLabel")]
    pub async fn update_wallet_transaction_label(
        &self,
        wallet_id: String,
        wallet_transaction_id: String,
        label: String,
    ) -> Result<WasmWalletTransaction, WasmError> {
        self.0
            .update_wallet_transaction_label(wallet_id, wallet_transaction_id, label)
            .await
            .map_err(|e| e.into())
            .map(|t| t.into())
    }

    #[wasm_bindgen(js_name = "deleteWalletTransaction")]
    pub async fn delete_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_transaction_id: String,
    ) -> Result<(), WasmError> {
        self.0
            .delete_wallet_transactions(wallet_id, wallet_transaction_id)
            .await
            .map_err(|e| e.into())
    }
}
