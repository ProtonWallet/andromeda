use andromeda_api::{
    wallet::{
        ApiEmailAddress, ApiWalletAccount, ApiWalletData, ApiWalletTransaction, CreateWalletAccountRequestBody,
        CreateWalletRequestBody, CreateWalletTransactionRequestBody, MigratedWallet, MigratedWalletAccount,
        MigratedWalletTransaction, TransactionType, WalletClient, WalletMigrateRequestBody, WalletTransactionFlag,
    },
    wallet_ext::WalletClientExt,
};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::{exchange_rate::WasmApiExchangeRate, settings::WasmFiatCurrencySymbol};
use crate::{
    bitcoin::types::derivation_path::WasmDerivationPath,
    common::{
        error::ErrorExt,
        types::{FromBool, WasmScriptType},
    },
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
    pub MigrationRequired: Option<u8>,
    pub Legacy: Option<u8>,
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
    pub ShowWalletRecovery: Option<bool>,
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
                MigrationRequired: value.Wallet.MigrationRequired,
                Legacy: value.Wallet.Legacy,
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
                ShowWalletRecovery: value.WalletSettings.ShowWalletRecovery,
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
    pub LastUsedIndex: u32,
    pub PoolSize: u32,
    pub Priority: u32,
    pub ScriptType: u8,
    pub StopGap: u16,
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

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmWalletAccountAddressData {
    pub Data: WasmApiEmailAddress,
}

impl From<ApiWalletAccount> for WasmApiWalletAccount {
    fn from(value: ApiWalletAccount) -> Self {
        WasmApiWalletAccount {
            WalletID: value.WalletID,
            FiatCurrency: value.FiatCurrency.into(),
            ID: value.ID,
            Label: value.Label,
            LastUsedIndex: value.LastUsedIndex,
            PoolSize: value.PoolSize,
            Priority: value.Priority,
            DerivationPath: value.DerivationPath,
            ScriptType: value.ScriptType,
            StopGap: value.StopGap,
            Addresses: value.Addresses.into_iter().map(|a| a.into()).collect::<Vec<_>>(),
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
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
    pub IsSuspicious: u8,
    pub IsPrivate: u8,
    pub IsAnonymous: Option<u8>,
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
            IsAnonymous: value.IsAnonymous,
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

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmMigratedWallet {
    /// Name of the wallet, encrypted
    pub Name: String,
    /// Encrypted user Id
    pub UserKeyID: String,
    /// Base64 encoded binary data
    pub WalletKey: String,
    /// Detached signature of the encrypted AES-GCM 256 key used to encrypt the
    /// mnemonic or public key, as armored PGP
    pub WalletKeySignature: String,
    /// Wallet mnemonic encrypted with the WalletKey, in base64 format
    pub Mnemonic: String,
    pub Fingerprint: String,
}

impl From<WasmMigratedWallet> for MigratedWallet {
    fn from(value: WasmMigratedWallet) -> Self {
        MigratedWallet {
            Name: value.Name,
            UserKeyID: value.UserKeyID,
            WalletKey: value.WalletKey,
            WalletKeySignature: value.WalletKeySignature,
            Mnemonic: value.Mnemonic,
            Fingerprint: value.Fingerprint,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmMigratedWalletAccount {
    // Wallet account ID
    pub ID: String,
    // Encrypted Label
    pub Label: String,
}

impl From<WasmMigratedWalletAccount> for MigratedWalletAccount {
    fn from(value: WasmMigratedWalletAccount) -> Self {
        MigratedWalletAccount {
            ID: value.ID,
            Label: value.Label,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmMigratedWalletAccountData {
    pub Data: WasmMigratedWalletAccount,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmMigratedWalletAccounts(pub Vec<WasmMigratedWalletAccountData>);

#[wasm_bindgen]
impl WasmMigratedWalletAccounts {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, account_data: WasmMigratedWalletAccount) {
        self.0.push(WasmMigratedWalletAccountData { Data: account_data })
    }
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmMigratedWalletTransaction {
    // Wallet ID
    pub ID: String,
    pub WalletAccountID: String,
    // encrypted transaction ID
    pub HashedTransactionID: Option<String>,
    // encrypted label
    pub Label: Option<String>,
}

impl From<WasmMigratedWalletTransaction> for MigratedWalletTransaction {
    fn from(value: WasmMigratedWalletTransaction) -> Self {
        MigratedWalletTransaction {
            ID: value.ID,
            WalletAccountID: value.WalletAccountID,
            HashedTransactionID: value.HashedTransactionID,
            Label: value.Label,
        }
    }
}
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmMigratedWalletTransactionData {
    pub Data: WasmMigratedWalletTransaction,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmMigratedWalletTransactions(pub Vec<WasmMigratedWalletTransactionData>);

#[wasm_bindgen]
impl WasmMigratedWalletTransactions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, account_data: WasmMigratedWalletTransaction) {
        self.0.push(WasmMigratedWalletTransactionData { Data: account_data })
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
pub struct WasmApiWalletAccountAddresses(pub Vec<WasmWalletAccountAddressData>);

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiWalletTransactions(pub Vec<WasmApiWalletTransactionData>);

#[wasm_bindgen]
impl WasmWalletClient {
    #[wasm_bindgen(js_name = "getWallets")]
    pub async fn get_wallets(&self) -> Result<WasmApiWalletsData, JsValue> {
        let wallets = self
            .0
            .get_wallets()
            .await
            .map_err(|e| e.to_js_error())
            .map(|wallets| wallets.into_iter().map(|wallet| wallet.into()).collect::<Vec<_>>())?;

        Ok(WasmApiWalletsData(wallets))
    }

    #[wasm_bindgen(js_name = "createWallet")]
    #[allow(clippy::too_many_arguments)]
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
        is_auto_created: Option<bool>,
    ) -> Result<WasmApiWalletData, JsValue> {
        let payload = CreateWalletRequestBody {
            Name: name,
            IsImported: u8::from_bool(is_imported),
            Type: wallet_type,
            HasPassphrase: u8::from_bool(has_passphrase),
            UserKeyID: user_key_id,
            WalletKey: wallet_key,
            WalletKeySignature: wallet_key_signature,
            Mnemonic: mnemonic,
            Fingerprint: fingerprint,
            PublicKey: public_key,
            IsAutoCreated: is_auto_created.map(u8::from_bool).unwrap_or(0),
        };

        self.0
            .create_wallet(payload)
            .await
            .map_err(|e| e.to_js_error())
            .map(|wallet| wallet.into())
    }

    #[wasm_bindgen(js_name = "migrate")]
    pub async fn migrate(
        &self,
        wallet_id: String,
        migrated_wallet: WasmMigratedWallet,
        migrated_wallet_accounts: WasmMigratedWalletAccounts,
        migrated_wallet_transactions: WasmMigratedWalletTransactions,
    ) -> Result<(), JsValue> {
        let payload = WalletMigrateRequestBody {
            Wallet: migrated_wallet.into(),
            WalletAccounts: migrated_wallet_accounts.0.into_iter().map(|v| v.Data.into()).collect(),
            WalletTransactions: migrated_wallet_transactions
                .0
                .into_iter()
                .map(|v| v.Data.into())
                .collect(),
        };

        self.0.migrate(wallet_id, payload).await.map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "updateWalletName")]
    pub async fn update_wallet_name(&self, wallet_id: String, name: String) -> Result<(), JsValue> {
        self.0
            .update_wallet_name(wallet_id, name)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "disableShowWalletRecovery")]
    pub async fn disable_show_wallet_recovery(&self, wallet_id: String) -> Result<(), JsValue> {
        self.0
            .disable_show_wallet_recovery(wallet_id)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "sendWalletAccountMetrics")]
    pub async fn send_wallet_account_metrics(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        has_positive_balance: bool,
    ) -> Result<(), JsValue> {
        self.0
            .send_wallet_account_metrics(wallet_id, wallet_account_id, has_positive_balance)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "deleteWallet")]
    pub async fn delete_wallets(&self, wallet_id: String) -> Result<(), JsValue> {
        self.0.delete_wallet(wallet_id).await.map_err(|e| e.to_js_error())?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "getWalletAccounts")]
    pub async fn get_wallet_accounts(&self, wallet_id: String) -> Result<WasmApiWalletAccounts, JsValue> {
        let wallet_accounts = self
            .0
            .get_wallet_accounts(wallet_id)
            .await
            .map_err(|e| e.to_js_error())?;

        let wallet_accounts: Result<Vec<WasmWalletAccountData>, JsValue> = wallet_accounts
            .into_iter()
            .map(|account| Ok(WasmWalletAccountData { Data: account.into() }))
            .collect();

        Ok(WasmApiWalletAccounts(wallet_accounts?))
    }

    #[wasm_bindgen(js_name = "getWalletAccountAddresses")]
    pub async fn get_wallet_account_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
    ) -> Result<WasmApiWalletAccountAddresses, JsValue> {
        let wallet_account_addresses = self
            .0
            .get_wallet_account_addresses(wallet_id, wallet_account_id)
            .await
            .map_err(|e| e.to_js_error())?;

        let wallet_account_addresses: Result<Vec<WasmWalletAccountAddressData>, JsValue> = wallet_account_addresses
            .into_iter()
            .map(|address| Ok(WasmWalletAccountAddressData { Data: address.into() }))
            .collect();

        Ok(WasmApiWalletAccountAddresses(wallet_account_addresses?))
    }

    #[wasm_bindgen(js_name = "createWalletAccount")]
    pub async fn create_wallet_account(
        &self,
        wallet_id: String,
        derivation_path: WasmDerivationPath,
        label: String,
        script_type: WasmScriptType,
    ) -> Result<WasmWalletAccountData, JsValue> {
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
    ) -> Result<WasmWalletAccountData, JsValue> {
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
    ) -> Result<WasmWalletAccountData, JsValue> {
        let account = self
            .0
            .update_wallet_account_label(wallet_id, wallet_account_id, label)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "updateWalletAccountsOrder")]
    pub async fn update_wallet_accounts_order(
        &self,
        wallet_id: String,
        wallet_account_ids: Vec<String>,
    ) -> Result<WasmApiWalletAccounts, JsValue> {
        let wallet_accounts = self
            .0
            .update_wallet_accounts_order(wallet_id, wallet_account_ids)
            .await
            .map_err(|e| e.to_js_error())?;

        let wallet_accounts: Result<Vec<WasmWalletAccountData>, JsValue> = wallet_accounts
            .into_iter()
            .map(|account| Ok(WasmWalletAccountData { Data: account.into() }))
            .collect();

        Ok(WasmApiWalletAccounts(wallet_accounts?))
    }

    #[wasm_bindgen(js_name = "updateWalletAccountLastUsedIndex")]
    pub async fn update_wallet_account_last_used_index(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        last_used_index: u32,
    ) -> Result<WasmWalletAccountData, JsValue> {
        let account = self
            .0
            .update_wallet_account_last_used_index(wallet_id, wallet_account_id, last_used_index)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "updateWalletAccountStopGap")]
    pub async fn update_wallet_account_stop_gap(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        stop_gap: u16,
    ) -> Result<WasmWalletAccountData, JsValue> {
        let account = self
            .0
            .update_wallet_account_stop_gap(wallet_id, wallet_account_id, stop_gap)
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
    ) -> Result<WasmWalletAccountData, JsValue> {
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
    ) -> Result<WasmWalletAccountData, JsValue> {
        let account = self
            .0
            .remove_email_address(wallet_id, wallet_account_id, email_address_id)
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmWalletAccountData { Data: account.into() })
    }

    #[wasm_bindgen(js_name = "deleteWalletAccount")]
    pub async fn delete_wallet_account(&self, wallet_id: String, wallet_account_id: String) -> Result<(), JsValue> {
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
    ) -> Result<WasmApiWalletTransactions, JsValue> {
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
    ) -> Result<WasmApiWalletTransactions, JsValue> {
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
    ) -> Result<WasmApiWalletTransactionData, JsValue> {
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
    ) -> Result<WasmApiWalletTransactionData, JsValue> {
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
    ) -> Result<WasmApiWalletTransactionData, JsValue> {
        self.0
            .update_wallet_transaction_hashed_txid(wallet_id, wallet_account_id, wallet_transaction_id, hash_txid)
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    #[wasm_bindgen(js_name = "updateExternalWalletTransactionSender")]
    pub async fn update_external_wallet_transaction_sender(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        sender: String,
    ) -> Result<WasmApiWalletTransactionData, JsValue> {
        self.0
            .update_external_wallet_transaction_sender(wallet_id, wallet_account_id, wallet_transaction_id, sender)
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    #[wasm_bindgen(js_name = "setWalletTransactionFlag")]
    pub async fn set_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WasmWalletTransactionFlag,
    ) -> Result<WasmApiWalletTransactionData, JsValue> {
        self.0
            .set_wallet_transaction_flag(wallet_id, wallet_account_id, wallet_transaction_id, flag.into())
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    #[wasm_bindgen(js_name = "deleteWalletTransactionFlag")]
    pub async fn delete_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WasmWalletTransactionFlag,
    ) -> Result<WasmApiWalletTransactionData, JsValue> {
        self.0
            .delete_wallet_transaction_flag(wallet_id, wallet_account_id, wallet_transaction_id, flag.into())
            .await
            .map_err(|e| e.to_js_error())
            .map(|t| WasmApiWalletTransactionData { Data: t.into() })
    }

    #[wasm_bindgen(js_name = "deleteWalletTransaction")]
    pub async fn delete_wallet_transaction(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
    ) -> Result<(), JsValue> {
        self.0
            .delete_wallet_transaction(wallet_id, wallet_account_id, wallet_transaction_id)
            .await
            .map_err(|e| e.to_js_error())
    }
}
