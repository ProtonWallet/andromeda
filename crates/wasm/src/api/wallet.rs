use andromeda_api::wallet::{
    ApiEmailAddress, ApiWalletAccount, ApiWalletData, ApiWalletTransaction, CreateWalletAccountRequestBody,
    CreateWalletRequestBody, CreateWalletTransactionRequestBody, TransactionType, WalletClient, WalletTransactionFlag,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::{exchange_rate::WasmApiExchangeRate, settings::WasmFiatCurrencySymbol};
use crate::{
    bitcoin::types::derivation_path::WasmDerivationPath,
    common::{error::ErrorExt, types::WasmScriptType},
};

#[wasm_bindgen]
#[derive(Clone)]
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
pub struct WasmApiWalletKey {
    pub WalletID: String,
    pub UserKeyID: String,
    pub WalletKey: String,
    pub WalletKeySignature: String,
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiWalletSettings {
    pub WalletID: String,
    pub HideAccounts: u8,
    pub InvoiceDefaultDescription: Option<String>,
    pub InvoiceExpirationTime: u64,
    pub MaxChannelOpeningFee: u64,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmApiWalletData {
    pub Wallet: WasmApiWallet,
    pub WalletKey: WasmApiWalletKey,
    pub WalletSettings: WasmApiWalletSettings,
}

impl From<ApiWalletData> for WasmApiWalletData {
    fn from(value: ApiWalletData) -> Self {
        WasmApiWalletData {
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
            WalletKey: WasmApiWalletKey {
                WalletID: value.WalletKey.WalletID,
                UserKeyID: value.WalletKey.UserKeyID,
                WalletKey: value.WalletKey.WalletKey,
                WalletKeySignature: value.WalletKey.WalletKeySignature,
            },
            WalletSettings: WasmApiWalletSettings {
                WalletID: value.WalletSettings.WalletID,
                HideAccounts: value.WalletSettings.HideAccounts,
                InvoiceDefaultDescription: value.WalletSettings.InvoiceDefaultDescription,
                InvoiceExpirationTime: value.WalletSettings.InvoiceExpirationTime,
                MaxChannelOpeningFee: value.WalletSettings.MaxChannelOpeningFee,
            },
        }
    }
}

#[wasm_bindgen]
impl WasmApiWalletData {
    #[wasm_bindgen]
    pub fn from_parts(wallet: WasmApiWallet, key: WasmApiWalletKey, settings: WasmApiWalletSettings) -> Self {
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
pub struct WasmApiEmailAddress {
    pub ID: String,
    pub Email: String,
}

impl From<ApiEmailAddress> for WasmApiEmailAddress {
    fn from(value: ApiEmailAddress) -> Self {
        WasmApiEmailAddress {
            ID: value.ID,
            Email: value.Email,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiWalletAccount {
    pub WalletID: String,
    pub FiatCurrency: WasmFiatCurrencySymbol,
    pub ID: String,
    pub DerivationPath: String,
    pub Label: String,
    pub ScriptType: u8,
    pub Addresses: Vec<WasmApiEmailAddress>,
}

// We need this wrapper because unfortunately, tsify doesn't support
// VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmWalletAccountData {
    pub Data: WasmApiWalletAccount,
}

impl From<ApiWalletAccount> for WasmApiWalletAccount {
    fn from(value: ApiWalletAccount) -> Self {
        WasmApiWalletAccount {
            WalletID: value.WalletID,
            FiatCurrency: value.FiatCurrency.into(),
            ID: value.ID,
            Label: value.Label,
            DerivationPath: value.DerivationPath,
            ScriptType: value.ScriptType,
            Addresses: value.Addresses.into_iter().map(|a| a.into()).collect::<Vec<_>>(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WasmTransactionType {
    NotSend,
    ProtonToProtonSend,
    ProtonToProtonReceive,
    ExternalSend,
    ExternalReceive,
    Unsupported,
}

impl From<TransactionType> for WasmTransactionType {
    fn from(value: TransactionType) -> Self {
        match value {
            TransactionType::NotSend => WasmTransactionType::NotSend,
            TransactionType::ProtonToProtonSend => WasmTransactionType::ProtonToProtonSend,
            TransactionType::ProtonToProtonReceive => WasmTransactionType::ProtonToProtonReceive,
            TransactionType::ExternalSend => WasmTransactionType::ExternalSend,
            TransactionType::ExternalReceive => WasmTransactionType::ExternalReceive,
            TransactionType::Unsupported => WasmTransactionType::Unsupported,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiWalletTransaction {
    pub ID: String,
    pub Type: Option<WasmTransactionType>,
    pub WalletID: String,
    pub WalletAccountID: Option<String>,
    pub Label: Option<String>,
    pub TransactionID: String,
    pub TransactionTime: String,
    pub IsSuspicious: bool,
    pub IsPrivate: bool,
    pub ExchangeRate: Option<WasmApiExchangeRate>,
    pub HashedTransactionID: Option<String>,
    pub Subject: Option<String>,
    pub Body: Option<String>,
    pub ToList: Option<String>,
    pub Sender: Option<String>,
}

impl From<ApiWalletTransaction> for WasmApiWalletTransaction {
    fn from(value: ApiWalletTransaction) -> Self {
        WasmApiWalletTransaction {
            ID: value.ID,
            Type: value.Type.map(|t| t.into()),
            WalletID: value.WalletID,
            WalletAccountID: value.WalletAccountID,
            Label: value.Label,
            TransactionID: value.TransactionID,
            TransactionTime: value.TransactionTime,
            IsSuspicious: value.IsSuspicious,
            IsPrivate: value.IsPrivate,
            ExchangeRate: value.ExchangeRate.map(|r| r.into()),
            HashedTransactionID: value.HashedTransactionID,
            Subject: value.Subject,
            Body: value.Body,
            ToList: value.ToList,
            Sender: value.Sender,
        }
    }
}

#[wasm_bindgen]
pub enum WasmWalletTransactionFlag {
    Suspicious,
    Private,
}

impl From<WasmWalletTransactionFlag> for WalletTransactionFlag {
    fn from(value: WasmWalletTransactionFlag) -> Self {
        match value {
            WasmWalletTransactionFlag::Suspicious => WalletTransactionFlag::Suspicious,
            WasmWalletTransactionFlag::Private => WalletTransactionFlag::Private,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmCreateWalletTransactionPayload {
    pub txid: String,
    pub hashed_txid: String,
    pub label: Option<String>,
    pub exchange_rate_id: Option<String>,
    pub transaction_time: Option<String>,
}

impl From<WasmCreateWalletTransactionPayload> for CreateWalletTransactionRequestBody {
    fn from(value: WasmCreateWalletTransactionPayload) -> Self {
        CreateWalletTransactionRequestBody {
            Label: value.label,
            HashedTransactionID: value.hashed_txid,
            TransactionID: value.txid,
            TransactionTime: value.transaction_time,
            ExchangeRateID: value.exchange_rate_id,
        }
    }
}

// We need this wrapper because unfortunately, tsify doesn't support
// VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmApiWalletTransactionData {
    pub Data: WasmApiWalletTransaction,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiWalletsData(pub Vec<WasmApiWalletData>);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiWalletAccounts(pub Vec<WasmWalletAccountData>);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiWalletTransactions(pub Vec<WasmApiWalletTransactionData>);

#[wasm_bindgen]
impl WasmWalletClient {
    #[wasm_bindgen(js_name = "getWallets")]
    pub async fn get_wallets(&self) -> Result<WasmApiWalletsData, js_sys::Error> {
        let wallets = self
            .0
            .get_wallets()
            .await
            .map_err(|e| e.to_js_error())
            .map(|wallets| wallets.into_iter().map(|wallet| wallet.into()).collect::<Vec<_>>())?;

        Ok(WasmApiWalletsData(wallets))
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
        wallet_key_signature: String,
        mnemonic: Option<String>,
        fingerprint: Option<String>,
        public_key: Option<String>,
    ) -> Result<WasmApiWalletData, js_sys::Error> {
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
            UserKeyID: user_key_id,
            WalletKey: wallet_key,
            WalletKeySignature: wallet_key_signature,
            Mnemonic: mnemonic,
            Fingerprint: fingerprint,
            PublicKey: public_key,
        };

        self.0
            .create_wallet(payload)
            .await
            .map_err(|e| e.to_js_error())
            .map(|wallet| wallet.into())
    }

    #[wasm_bindgen(js_name = "updateWalletName")]
    pub async fn update_wallet_name(&self, wallet_id: String, name: String) -> Result<(), js_sys::Error> {
        self.0
            .update_wallet_name(wallet_id, name)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteWallet")]
    pub async fn delete_wallets(&self, wallet_id: String) -> Result<(), js_sys::Error> {
        self.0.delete_wallet(wallet_id).await.map_err(|e| e.to_js_error())?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "getWalletAccounts")]
    pub async fn get_wallet_accounts(&self, wallet_id: String) -> Result<WasmApiWalletAccounts, js_sys::Error> {
        let wallet_accounts = self
            .0
            .get_wallet_accounts(wallet_id)
            .await
            .map_err(|e| e.to_js_error())?;

        let wallet_accounts: Result<Vec<WasmWalletAccountData>, js_sys::Error> = wallet_accounts
            .into_iter()
            .map(|account| Ok(WasmWalletAccountData { Data: account.into() }))
            .collect();

        Ok(WasmApiWalletAccounts(wallet_accounts?))
    }

    #[wasm_bindgen(js_name = "createWalletAccount")]
    pub async fn create_wallet_account(
        &self,
        wallet_id: String,
        derivation_path: WasmDerivationPath,
        label: String,
        script_type: WasmScriptType,
    ) -> Result<WasmWalletAccountData, js_sys::Error> {
        let payload = CreateWalletAccountRequestBody {
            DerivationPath: derivation_path.inner().to_string(),
            Label: label,
            ScriptType: script_type.into(),
        };

        let account = self
            .0
            .create_wallet_account(wallet_id, payload)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "updateWalletAccountFiatCurrency")]
    pub async fn update_wallet_account_fiat_currency(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        symbol: WasmFiatCurrencySymbol,
    ) -> Result<WasmWalletAccountData, js_sys::Error> {
        let account = self
            .0
            .update_wallet_account_fiat_currency(wallet_id, wallet_account_id, symbol.into())
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "updateWalletAccountLabel")]
    pub async fn update_wallet_account_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        label: String,
    ) -> Result<WasmWalletAccountData, js_sys::Error> {
        let account = self
            .0
            .update_wallet_account_label(wallet_id, wallet_account_id, label)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "addEmailAddress")]
    pub async fn add_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        email_address_id: String,
    ) -> Result<WasmWalletAccountData, js_sys::Error> {
        let account = self
            .0
            .add_email_address(wallet_id, wallet_account_id, email_address_id)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "removeEmailAddress")]
    pub async fn remove_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        email_address_id: String,
    ) -> Result<WasmWalletAccountData, js_sys::Error> {
        let account = self
            .0
            .remove_email_address(wallet_id, wallet_account_id, email_address_id)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "deleteWalletAccount")]
    pub async fn delete_wallet_account(
        &self,
        wallet_id: String,
        wallet_account_id: String,
    ) -> Result<(), js_sys::Error> {
        self.0
            .delete_wallet_account(wallet_id, wallet_account_id)
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "getWalletTransactions")]
    pub async fn get_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
        hashed_txids: Option<Vec<String>>,
    ) -> Result<WasmApiWalletTransactions, js_sys::Error> {
        let wallet_transactions = self
            .0
            .get_wallet_transactions(wallet_id, wallet_account_id, hashed_txids)
            .await
            .map_err(|e| e.to_js_error())
            .map(|transactions| {
                transactions
                    .into_iter()
                    .map(|t| WasmApiWalletTransactionData { Data: t.into() })
                    .collect::<Vec<_>>()
            })?;

        Ok(WasmApiWalletTransactions(wallet_transactions))
    }

    #[wasm_bindgen(js_name = "getWalletTransactionsToHash")]
    pub async fn get_wallet_transactions_to_hash(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
    ) -> Result<WasmApiWalletTransactions, js_sys::Error> {
        let wallet_transactions = self
            .0
            .get_wallet_transactions_to_hash(wallet_id, wallet_account_id)
            .await
            .map_err(|e| e.to_js_error())
            .map(|transactions| {
                transactions
                    .into_iter()
                    .map(|t| WasmApiWalletTransactionData { Data: t.into() })
                    .collect::<Vec<_>>()
            })?;

        Ok(WasmApiWalletTransactions(wallet_transactions))
    }

    #[wasm_bindgen(js_name = "createWalletTransaction")]
    pub async fn create_wallet_transaction(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        payload: WasmCreateWalletTransactionPayload,
    ) -> Result<WasmApiWalletTransactionData, js_sys::Error> {
        self.0
            .create_wallet_transaction(wallet_id, wallet_account_id, payload.into())
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    #[wasm_bindgen(js_name = "updateWalletTransactionLabel")]
    pub async fn update_wallet_transaction_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        label: String,
    ) -> Result<WasmApiWalletTransactionData, js_sys::Error> {
        self.0
            .update_wallet_transaction_label(wallet_id, wallet_account_id, wallet_transaction_id, label)
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    #[wasm_bindgen(js_name = "updateWalletTransactionHashedTxId")]
    pub async fn update_wallet_transaction_hashed_txid(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        hash_txid: String,
    ) -> Result<WasmApiWalletTransactionData, js_sys::Error> {
        self.0
            .update_wallet_transaction_hashed_txid(wallet_id, wallet_account_id, wallet_transaction_id, hash_txid)
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    pub async fn update_external_wallet_transaction_sender(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        sender: String,
    ) -> Result<WasmApiWalletTransactionData, js_sys::Error> {
        self.0
            .update_external_wallet_transaction_sender(wallet_id, wallet_account_id, wallet_transaction_id, sender)
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    pub async fn set_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WasmWalletTransactionFlag,
    ) -> Result<WasmApiWalletTransactionData, js_sys::Error> {
        self.0
            .set_wallet_transaction_flag(wallet_id, wallet_account_id, wallet_transaction_id, flag.into())
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    pub async fn delete_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WasmWalletTransactionFlag,
    ) -> Result<WasmApiWalletTransactionData, js_sys::Error> {
        self.0
            .delete_wallet_transaction_flag(wallet_id, wallet_account_id, wallet_transaction_id, flag.into())
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    #[wasm_bindgen(js_name = "deleteWalletTransaction")]
    pub async fn delete_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
    ) -> Result<(), js_sys::Error> {
        self.0
            .delete_wallet_transactions(wallet_id, wallet_account_id, wallet_transaction_id)
            .await
            .map_err(|e| e.to_js_error())
    }
}
