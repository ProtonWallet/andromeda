use crate::common::error::WasmError;
use andromeda_api::wallet::{
    Account, CreateWalletAccountRequestBody, CreateWalletRequestBody, CreateWalletTransactionRequestBody, WalletClient,
    WalletData, WalletTransaction,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmWalletClient(WalletClient);

impl From<WalletClient> for WasmWalletClient {
    fn from(value: WalletClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
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
    pub PublicKey: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[allow(non_snake_case)]
pub struct WasmCreateWalletRequestBody {
    pub Name: String,
    pub IsImported: u8,
    pub Type: u8,
    pub HasPassphrase: u8,
    pub UserKeyId: String,
    pub WalletKey: String,
    pub Mnemonic: Option<String>,
    pub PublicKey: Option<String>,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmWalletKey {
    pub UserKeyID: String,
    pub WalletKey: String,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmWalletAccount {
    pub ID: String,
    pub DerivationPath: String,
    pub Label: String,
    pub ScriptType: u8,
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
#[allow(non_snake_case)]
pub struct WasmCreateWalletAccountRequestBody {
    pub DerivationPath: String,
    pub Label: String,
    pub ScriptType: u8,
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
#[allow(non_snake_case)]
pub struct WasmCreateWalletTransactionRequestBody {
    /// encrypted Base64 encoded binary data
    pub Label: String,
    /// encrypted Base64 encoded binary data
    pub TransactionID: String,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmWalletDataArray(pub Vec<WasmWalletData>);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmWalletAccountArray(pub Vec<WasmWalletAccount>);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmWalletTransactionArray(pub Vec<WasmWalletTransaction>);

#[wasm_bindgen]
impl WasmWalletClient {
    #[wasm_bindgen(js_name="getWallets")]
    pub async fn get_wallets(&self) -> Result<WasmWalletDataArray, WasmError> {
        let wallets = self
            .0
            .get_wallets()
            .await
            .map_err(|e| e.into())
            .map(|wallets| wallets.into_iter().map(|wallet| wallet.into()).collect::<Vec<_>>())?;

        Ok(WasmWalletDataArray(wallets))
    }

    #[wasm_bindgen(js_name="createWallet")]
    pub async fn create_wallet(&self, payload: WasmCreateWalletRequestBody) -> Result<WasmWalletData, WasmError> {
        let payload = CreateWalletRequestBody {
            Name: payload.Name,
            IsImported: payload.IsImported,
            Type: payload.Type,
            HasPassphrase: payload.HasPassphrase,
            UserKeyId: payload.UserKeyId,
            WalletKey: payload.WalletKey,
            Mnemonic: payload.Mnemonic,
            PublicKey: payload.PublicKey,
        };

        self.0
            .create_wallet(payload)
            .await
            .map_err(|e| e.into())
            .map(|wallet| wallet.into())
    }

    #[wasm_bindgen(js_name="getWalletAccounts")]
    pub async fn get_wallet_accounts(&self, wallet_id: String) -> Result<WasmWalletAccountArray, WasmError> {
        let wallet_accounts = self
            .0
            .get_wallet_accounts(wallet_id)
            .await
            .map_err(|e| e.into())
            .map(|accounts| accounts.into_iter().map(|account| account.into()).collect::<Vec<_>>())?;

        Ok(WasmWalletAccountArray(wallet_accounts))
    }

    #[wasm_bindgen(js_name="createWalletAccount")]
    pub async fn create_wallet_account(
        &self,
        wallet_id: String,
        payload: WasmCreateWalletAccountRequestBody,
    ) -> Result<WasmWalletAccount, WasmError> {
        let payload = CreateWalletAccountRequestBody {
            DerivationPath: payload.DerivationPath,
            Label: payload.Label,
            ScriptType: payload.ScriptType,
        };

        self.0
            .create_wallet_account(wallet_id, payload)
            .await
            .map_err(|e| e.into())
            .map(|account| account.into())
    }

    #[wasm_bindgen(js_name="updateWalletAccountLabel")]
    pub async fn update_wallet_account_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        label: String,
    ) -> Result<WasmWalletAccount, WasmError> {
        self.0
            .update_wallet_account_label(wallet_id, wallet_account_id, label)
            .await
            .map_err(|e| e.into())
            .map(|account| account.into())
    }

    #[wasm_bindgen(js_name="deleteWalletAccount")]
    pub async fn delete_wallet_account(&self, wallet_id: String, wallet_account_id: String) -> Result<(), WasmError> {
        self.0
            .delete_wallet_account(wallet_id, wallet_account_id)
            .await
            .map_err(|e| e.into())
    }

    #[wasm_bindgen(js_name="getWalletTransactions")]
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

    #[wasm_bindgen(js_name="createWalletTransaction")]
    pub async fn create_wallet_transaction(
        &self,
        wallet_id: String,
        payload: WasmCreateWalletTransactionRequestBody,
    ) -> Result<WasmWalletTransaction, WasmError> {
        let payload = CreateWalletTransactionRequestBody {
            Label: payload.Label,
            TransactionID: payload.TransactionID,
        };

        self.0
            .create_wallet_transaction(wallet_id, payload)
            .await
            .map_err(|e| e.into())
            .map(|t| t.into())
    }

    #[wasm_bindgen(js_name="updateWalletTransactionLabel")]
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

    #[wasm_bindgen(js_name="deleteWalletTransaction")]
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
