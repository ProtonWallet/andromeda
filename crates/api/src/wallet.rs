use core::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::BASE_WALLET_API_V1;
use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    exchange_rate::ApiExchangeRate,
    settings::FiatCurrencySymbol,
    ProtonWalletApiClient,
};

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ApiWallet {
    pub ID: String,
    /// Name of the wallet
    pub Name: String,
    /// 0 if the wallet is created with Proton Wallet
    pub IsImported: u8,
    /// Priority of the wallet (0 is main wallet)
    pub Priority: u8,
    /// 1 is onchain, 2 is lightning
    pub Type: u8,
    /// 1 if the wallet has a passphrase. We don't store it but clients need to
    /// request on first wallet access.
    pub HasPassphrase: u8,
    /// 1 means disabled
    pub Status: u8,
    /// Wallet mnemonic encrypted with the WalletKey, in base64 format
    pub Mnemonic: Option<String>,
    // Unique identifier of the mnemonic, using the first 4 bytes of the master public key hash
    pub Fingerprint: Option<String>,
    /// Wallet master public key encrypted with the WalletKey, in base64 format.
    /// Only allows fetching coins owned by wallet, no spending allowed.
    pub PublicKey: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct CreateWalletRequestBody {
    /// Name of the wallet
    pub Name: String,
    /// 0 if the wallet is created with Proton Wallet
    pub IsImported: u8,
    /// 1 is onchain, 2 is lightning
    pub Type: u8,
    /// 1 if the wallet has a passphrase. We don't store it but clients need to
    /// request on first wallet access.
    pub HasPassphrase: u8,
    /// Encrypted user Id
    pub UserKeyID: String,
    /// Base64 encoded binary data
    pub WalletKey: String,
    /// Detached signature of the encrypted AES-GCM 256 key used to encrypt the
    /// mnemonic or public key, as armored PGP
    pub WalletKeySignature: String,
    /// Wallet mnemonic encrypted with the WalletKey, in base64 format
    pub Mnemonic: Option<String>,
    // Unique identifier of the mnemonic, using the first 4 bytes of the master public key hash, required if Mnemonic
    // is provided
    pub Fingerprint: Option<String>,
    /// Wallet master public key encrypted with the WalletKey, in base64 format.
    /// Only allows fetching coins owned by wallet, no spending allowed.
    pub PublicKey: Option<String>,
    /// Flag that indicates the wallet is created from auto creation. 0 for no,
    /// 1 for yes
    pub IsAutoCreated: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletKey {
    pub WalletID: String,
    pub UserKeyID: String,
    pub WalletKey: String,
    pub WalletKeySignature: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletSettings {
    pub WalletID: String,
    pub HideAccounts: u8,
    pub InvoiceDefaultDescription: Option<String>,
    pub InvoiceExpirationTime: u64,
    pub MaxChannelOpeningFee: u64,
    pub ShowWalletRecovery: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletData {
    pub Wallet: ApiWallet,
    pub WalletKey: ApiWalletKey,
    pub WalletSettings: ApiWalletSettings,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetWalletsResponseBody {
    pub Code: u16,
    pub Wallets: Vec<ApiWalletData>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct CreateWalletResponseBody {
    pub Code: u16,
    pub Wallet: ApiWallet,
    pub WalletKey: ApiWalletKey,
    pub WalletSettings: ApiWalletSettings,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletNameRequestBody {
    pub Name: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletNameResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Wallet: ApiWallet,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletAccount {
    pub ID: String,
    pub WalletID: String,
    pub FiatCurrency: FiatCurrencySymbol,
    pub DerivationPath: String,
    pub Label: String,
    pub LastUsedIndex: u32,
    pub PoolSize: u32,
    pub Priority: u32,
    pub ScriptType: u8,
    pub Addresses: Vec<ApiEmailAddress>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiEmailAddress {
    pub ID: String,
    pub Email: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetWalletAccountsResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Accounts: Vec<ApiWalletAccount>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateWalletAccountRequestBody {
    pub DerivationPath: String,
    pub Label: String,
    pub ScriptType: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct CreateWalletAccountResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Account: ApiWalletAccount,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletAccountFiatCurrencyRequestBody {
    pub Symbol: FiatCurrencySymbol,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletAccountLabelRequestBody {
    pub Label: String,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletAccountLastUsedIndexRequestBody {
    pub LastUsedIndex: u32,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletAccountsOrderRequestBody {
    pub WalletAccountIDs: Vec<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct AddEmailAddressRequestBody {
    pub AddressID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletAccountResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Account: ApiWalletAccount,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletAccountsOrderResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub Accounts: Vec<ApiWalletAccount>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DeleteWalletAccountResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
}

#[derive(Deserialize_repr, Serialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum TransactionType {
    NotSend = 0,
    ProtonToProtonSend = 1,
    ProtonToProtonReceive = 2,
    ExternalSend = 3,
    ExternalReceive = 4,
    #[serde(other)]
    Unsupported,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletTransaction {
    pub ID: String,
    pub Type: Option<TransactionType>, // TODO: this should be made non-nullable once API is ready
    pub WalletID: String,
    pub WalletAccountID: Option<String>, // TODO this should be non-nullable
    pub Label: Option<String>,
    pub TransactionID: String,
    pub TransactionTime: String,
    pub IsSuspicious: u8,
    pub IsPrivate: u8,
    pub ExchangeRate: Option<ApiExchangeRate>,
    pub HashedTransactionID: Option<String>,
    pub Subject: Option<String>,
    pub Body: Option<String>,
    pub ToList: Option<String>,
    pub Sender: Option<String>,
}

pub enum WalletTransactionFlag {
    Suspicious,
    Private,
}

impl fmt::Display for WalletTransactionFlag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            WalletTransactionFlag::Suspicious => "suspicious",
            WalletTransactionFlag::Private => "private",
        };
        write!(f, "{}", output)
    }
}

const HASHED_TRANSACTION_ID_KEY: &str = "HashedTransactionIDs[]";

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetWalletTransactionsResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletTransactions: Vec<ApiWalletTransaction>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateWalletTransactionRequestBody {
    /// Encrypted with user key
    pub TransactionID: String,
    /// Hmac(walletKey, txid) and base64 encoded
    pub HashedTransactionID: String,
    /// Encrypted with wallet key and base64 encoded
    pub Label: Option<String>,
    /// Id of the exchange rate to use with this transaction
    pub ExchangeRateID: Option<String>,
    /// Unix timestamp of when the transaction got created in Proton Wallet or
    /// confirmed in blockchain for incoming ones
    pub TransactionTime: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct CreateWalletTransactionResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletTransaction: ApiWalletTransaction,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletTransactionResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletTransaction: ApiWalletTransaction,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletTransactionLabelRequestBody {
    /// encrypted Base64 encoded binary data
    pub Label: String,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletTransactionHashedTxidRequestBody {
    /// Hmac(walletKey, txid) and base64 encoded
    pub HashedTransactionID: String,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletTransactionExternalSenderRequestBody {
    /// An armored PGP Message
    pub Sender: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DeleteWalletTransactionResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletSettingsResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletSettings: ApiWalletSettings,
}

#[derive(Clone)]
pub struct WalletClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for WalletClient {
    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        &self.api_client
    }

    fn base_url(&self) -> &str {
        BASE_WALLET_API_V1
    }

    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }
}

impl WalletClient {
    pub async fn get_wallets(&self) -> Result<Vec<ApiWalletData>, Error> {
        let request = self.get("wallets");
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetWalletsResponseBody>()?;
        Ok(parsed.Wallets)
    }

    pub async fn create_wallet(&self, payload: CreateWalletRequestBody) -> Result<ApiWalletData, Error> {
        let request = self.post("wallets").body_json(payload)?;
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<CreateWalletResponseBody>()?;

        Ok(ApiWalletData {
            Wallet: parsed.Wallet,
            WalletKey: parsed.WalletKey,
            WalletSettings: parsed.WalletSettings,
        })
    }

    pub async fn update_wallet_name(&self, wallet_id: String, name: String) -> Result<ApiWallet, Error> {
        let payload = UpdateWalletNameRequestBody { Name: name };
        let request = self.put(format!("wallets/{}/name", wallet_id)).body_json(payload)?;
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletNameResponseBody>()?;
        Ok(parsed.Wallet)
    }

    pub async fn delete_wallet(&self, wallet_id: String) -> Result<(), Error> {
        let request = self.delete(format!("wallets/{}", wallet_id));
        let response = self.api_client.send(request).await?;
        response.parse_response::<DeleteWalletAccountResponseBody>()?;

        Ok(())
    }

    pub async fn get_wallet_accounts(&self, wallet_id: String) -> Result<Vec<ApiWalletAccount>, Error> {
        let request = self.get(format!("wallets/{}/accounts", wallet_id));
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetWalletAccountsResponseBody>()?;

        Ok(parsed.Accounts)
    }

    pub async fn create_wallet_account(
        &self,
        wallet_id: String,
        payload: CreateWalletAccountRequestBody,
    ) -> Result<ApiWalletAccount, Error> {
        let request = self
            .post(format!("wallets/{}/accounts", wallet_id))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<CreateWalletAccountResponseBody>()?;

        Ok(parsed.Account)
    }

    pub async fn update_wallet_account_fiat_currency(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        fiat_currency_symbol: FiatCurrencySymbol,
    ) -> Result<ApiWalletAccount, Error> {
        let payload = UpdateWalletAccountFiatCurrencyRequestBody {
            Symbol: fiat_currency_symbol,
        };
        let request = self
            .put(format!(
                "wallets/{}/accounts/{}/currency/fiat",
                wallet_id, wallet_account_id
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletAccountResponseBody>()?;

        Ok(parsed.Account)
    }

    pub async fn update_wallet_account_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        label: String,
    ) -> Result<ApiWalletAccount, Error> {
        let payload = UpdateWalletAccountLabelRequestBody { Label: label };

        let request = self
            .put(format!("wallets/{}/accounts/{}/label", wallet_id, wallet_account_id))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletAccountResponseBody>()?;

        Ok(parsed.Account)
    }

    pub async fn update_wallet_accounts_order(
        &self,
        wallet_id: String,
        wallet_account_ids: Vec<String>,
    ) -> Result<Vec<ApiWalletAccount>, Error> {
        let payload = UpdateWalletAccountsOrderRequestBody {
            WalletAccountIDs: wallet_account_ids,
        };

        let request = self
            .put(format!("wallets/{}/accounts/order", wallet_id))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletAccountsOrderResponseBody>()?;

        Ok(parsed.Accounts)
    }

    pub async fn add_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        address_id: String,
    ) -> Result<ApiWalletAccount, Error> {
        let payload = AddEmailAddressRequestBody { AddressID: address_id };

        let request = self
            .put(format!(
                "wallets/{}/accounts/{}/addresses/email",
                wallet_id, wallet_account_id
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletAccountResponseBody>()?;

        Ok(parsed.Account)
    }

    pub async fn update_wallet_account_last_used_index(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        last_used_index: u32,
    ) -> Result<ApiWalletAccount, Error> {
        let payload = UpdateWalletAccountLastUsedIndexRequestBody {
            LastUsedIndex: last_used_index,
        };

        let request = self
            .put(format!(
                "wallets/{}/accounts/{}/lastUsedIndex",
                wallet_id, wallet_account_id
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletAccountResponseBody>()?;

        Ok(parsed.Account)
    }

    pub async fn remove_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        address_id: String,
    ) -> Result<ApiWalletAccount, Error> {
        let request = self.delete(format!(
            "wallets/{}/accounts/{}/addresses/email/{}",
            wallet_id, wallet_account_id, address_id
        ));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletAccountResponseBody>()?;

        Ok(parsed.Account)
    }

    pub async fn delete_wallet_account(&self, wallet_id: String, wallet_account_id: String) -> Result<(), Error> {
        let request = self.delete(format!("wallets/{}/accounts/{}", wallet_id, wallet_account_id));
        let response = self.api_client.send(request).await?;
        response.parse_response::<DeleteWalletAccountResponseBody>()?;

        Ok(())
    }

    pub async fn get_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
        hashed_txids: Option<Vec<String>>,
    ) -> Result<Vec<ApiWalletTransaction>, Error> {
        let mut request = self.get(match wallet_account_id {
            Some(wallet_account_id) => {
                format!("wallets/{}/accounts/{}/transactions", wallet_id, wallet_account_id)
            }
            None => format!("wallets/{}/transactions", wallet_id),
        });

        for txid in hashed_txids.unwrap_or_default() {
            request = request.query((HASHED_TRANSACTION_ID_KEY, txid));
        }
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetWalletTransactionsResponseBody>()?;

        Ok(parsed.WalletTransactions)
    }

    pub async fn get_wallet_transactions_to_hash(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
    ) -> Result<Vec<ApiWalletTransaction>, Error> {
        let request = self.get(match wallet_account_id {
            Some(wallet_account_id) => {
                format!(
                    "wallets/{}/accounts/{}/transactions/to-hash",
                    wallet_id, wallet_account_id
                )
            }
            None => format!("wallets/{}/transactions/to-hash", wallet_id),
        });

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetWalletTransactionsResponseBody>()?;

        Ok(parsed.WalletTransactions)
    }

    pub async fn create_wallet_transaction(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        payload: CreateWalletTransactionRequestBody,
    ) -> Result<ApiWalletTransaction, Error> {
        let request = self
            .post(format!(
                "wallets/{}/accounts/{}/transactions",
                wallet_id, wallet_account_id
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<CreateWalletTransactionResponseBody>()?;

        Ok(parsed.WalletTransaction)
    }

    pub async fn update_wallet_transaction_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        label: String,
    ) -> Result<ApiWalletTransaction, Error> {
        let payload = UpdateWalletTransactionLabelRequestBody { Label: label };

        let request = self
            .put(format!(
                "wallets/{}/accounts/{}/transactions/{}/label",
                wallet_id, wallet_account_id, wallet_transaction_id
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletTransactionResponseBody>()?;

        Ok(parsed.WalletTransaction)
    }

    pub async fn update_wallet_transaction_hashed_txid(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        hash_txid: String,
    ) -> Result<ApiWalletTransaction, Error> {
        let payload = UpdateWalletTransactionHashedTxidRequestBody {
            HashedTransactionID: hash_txid,
        };

        let request = self
            .put(format!(
                "wallets/{}/accounts/{}/transactions/{}/hash",
                wallet_id, wallet_account_id, wallet_transaction_id
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletTransactionResponseBody>()?;

        Ok(parsed.WalletTransaction)
    }

    pub async fn update_external_wallet_transaction_sender(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        sender: String,
    ) -> Result<ApiWalletTransaction, Error> {
        let payload = UpdateWalletTransactionExternalSenderRequestBody { Sender: sender };

        let request = self
            .put(format!(
                "wallets/{}/accounts/{}/transactions/{}/sender",
                wallet_id, wallet_account_id, wallet_transaction_id
            ))
            .body_json(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletTransactionResponseBody>()?;

        Ok(parsed.WalletTransaction)
    }

    pub async fn set_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WalletTransactionFlag,
    ) -> Result<ApiWalletTransaction, Error> {
        let request = self.put(format!(
            "wallets/{}/accounts/{}/transactions/{}/{}",
            wallet_id,
            wallet_account_id,
            wallet_transaction_id,
            flag.to_string()
        ));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletTransactionResponseBody>()?;

        Ok(parsed.WalletTransaction)
    }

    pub async fn delete_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WalletTransactionFlag,
    ) -> Result<ApiWalletTransaction, Error> {
        let request = self.delete(format!(
            "wallets/{}/accounts/{}/transactions/{}/{}",
            wallet_id,
            wallet_account_id,
            wallet_transaction_id,
            flag.to_string()
        ));
        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletTransactionResponseBody>()?;

        Ok(parsed.WalletTransaction)
    }

    pub async fn delete_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
    ) -> Result<(), Error> {
        let request = self.delete(format!(
            "wallets/{}/accounts/{}/transactions/{}",
            wallet_id, wallet_account_id, wallet_transaction_id
        ));
        let response = self.api_client.send(request).await?;
        response.parse_response::<DeleteWalletTransactionResponseBody>()?;

        Ok(())
    }

    pub async fn disable_show_wallet_recovery(&self, wallet_id: String) -> Result<ApiWalletSettings, Error> {
        let request = self.put(format!("wallets/{}/settings/show-wallet-recovery/disable", wallet_id));

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateWalletSettingsResponseBody>()?;

        Ok(parsed.WalletSettings)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use andromeda_common::ScriptType;
    use bitcoin::bip32::DerivationPath;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::{
        CreateWalletAccountRequestBody, CreateWalletRequestBody, CreateWalletTransactionRequestBody, WalletClient,
    };
    use crate::{
        core::ApiClient,
        error::Error,
        tests::utils::{common_api_client, setup_test_connection_arc},
        BASE_WALLET_API_V1,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_wallets() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let result = client.get_wallets().await;
        println!("request done: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_create_wallet() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let payload = CreateWalletRequestBody {
            Name: String::from("yehAfGZvTxIBEu0lmlajrm3ewUZaF6eYr6nPBsZW5mU2KIGy2BK9FbsJ1wMNGolw4muTOq6+20GTOdRf"),
            Type: 1,
            HasPassphrase: 0,
            IsImported: 0,
            Mnemonic: Some(String::from("03sX/gGsT+3iZY4lAaPcza9J4vFTSc8UOrkmLuJkwl1TXVBmlI2hL0nzEBIt/MF7Pha3/Pby672E1lEPp81oF4W+3hHJABSqM3rZarDmpBGFU3HPTcyY3eenkC/DeUlp+gHfM9Rg2w==")),
            Fingerprint: Some(String::from("49707e7a")),
            PublicKey: None,
            UserKeyID: String::from("4Xi8TArBe1WYfrFoJF5_wIDF0shMe5ACAqOArU6hjpUNoC0O0c_Zu5Afz11gGU1eeDu5Aanp_EUkpd44kjQ2lg=="),
            WalletKey: String::from("-----BEGIN PGP MESSAGE-----\nVersion: ProtonMail\n\nwcBMA38ULORPpTD1AQgAgL+aR4cwCD+QKrW8XGlBoQv/e4sei9MFkLqoolu4\ncCoQIZXKBt6rroAgQaccwXngiTDrXELkAu2Bnjh6r5KakVu7cPyqjsIF3xjr\naSxWOZ0TcsmBNSgFgkITnNKrd4l9XKfMqCshII+mGVGb4r84glhLokMFU1xU\n5WcKGSry/oomDiyClBDnxdHr/sUNuj169uJz//uAMHQuFXqNtZ1wgwDlGCUL\nAZy5kquoSYZzSDksMj8TveIlV/HLQsFowBYgQks5FZm628Ufl/AY0F7zvxPZ\n359IAANyOi58RsX6U8500/moYd7S4aB4bRgbvUthPYOc5EAaj3I5dIphyy70\nbdJRAcf40LTwF1xOkNhIt5lEh3QAz1QxsV4miYJBbigZz0vCDyyiP/VuuexN\nb+atelhAp4ORS8j4GAe7BjXD4RFBG4avREjytzBd78tm4WitP4PY\n=ZA0x\n-----END PGP MESSAGE-----\n"),
            WalletKeySignature: String::from("-----BEGIN PGP SIGNATURE-----\nVersion: ProtonMail\n\nwsCYBAABCgBMBQJmVtLgCRAEzZ3CX7rlCRYhBFNy3nIbmXFRgnNYHgTNncJf\nuuUJJJSAAAAAABEACmNvbnRleHRAcHJvdG9uLmNod2FsbGV0LmtleQAAmmQH\n/3rVCYilw5rmF1BQkgR23oE5DrfYOKdcFbQvIqXq4in2BwVMWzcojZsxD4GC\nOHCaaC61TnaELHoy8waQzzNSEmydi3MpVuryUEuqlC7C9fwZLYDMrDXKPJcA\nGNmAnj80iMkZrCn00/fMP2CvIKiYhrEbjH1KHWxceGmm4oMpD7na1h9zMVxa\ni4DL2KZtW4vcrvYNlrUjFwCLenBPa1CBJ0abi4n8htUykjWHoJvYhPrm1QAS\ns96wsMFtbwMoLlKQzTxldzF/jS9H5RFl0DfADQhMkipAVKj1qsUgLB3BcFcD\nNeIP4uGLgqKGAKAeq+HX3NDKuvoSAFb4dKsIQuN2doQ=\n=gFPX\n-----END PGP SIGNATURE-----\n"),
            IsAutoCreated: 0
        };

        let result = client.create_wallet(payload).await;
        println!("request done: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_name() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .update_wallet_name(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from("yehAfGZvTxIBEu0lmlajrm3ewUZaF6eYr6nPBsZW5mU2KIGy2BK9FbsJ1wMNGolw4muTOq6+20GTOdRf"),
            )
            .await;
        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let wallet = client
            .delete_wallet(String::from(
                "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
            ))
            .await;

        println!("request done: {:?}", wallet);
        assert!(wallet.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_create_wallet_account() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let payload = CreateWalletAccountRequestBody {
            DerivationPath: DerivationPath::from_str("m/44'/1'/0'").unwrap().to_string(),
            Label: String::from("TXkgdGVzdCB3YWxsZXQgYWNjb3VudA=="),
            ScriptType: ScriptType::NativeSegwit.into(),
        };

        let res = client
            .create_wallet_account(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                payload,
            )
            .await;
        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_accounts() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .get_wallet_accounts(String::from(
                "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
            ))
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet_account() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .delete_wallet_account(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                // replace the account id with the one you want to delete
                String::from(
                    "_gsDVeX4osuFvPSlszWb-hGvo7d9poBm58MNxvvC2mmG2F1rfM72IqG3hJvGlgMqRHAMyXGgJCI0J8gfukLlXQ==",
                ),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_account_label() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .update_wallet_account_label(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from("QW5vdGhlciB0ZXN0IHdhbGxldCBhY2NvdW50IFhZWg=="),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_accounts_order() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .update_wallet_accounts_order(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                vec![
                    String::from(
                        "kBZYBzgHWtjW5igU33BXqwVZ66GBdJi4ycXPzZjyUmp840-O2yXyNEO0ayRveZKNnASS_btzUY-WkI_mcvNuOg==",
                    ),
                    String::from(
                        "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                    ),
                ],
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_account_last_used_index() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .update_wallet_account_last_used_index(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                666,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_by_wallet() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .get_wallet_transactions(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                None,
                None,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_by_wallet_account() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .get_wallet_transactions(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                Some(String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                )),
                None,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_with_query_param() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .get_wallet_transactions(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                None,
                Some(vec![
                    String::from("k5WX0lOyT6Xe3h14f1A+fxZ47owxcjQFkGQy72tAXeQ="),
                    String::from("ZxIeKb4btZCywLEMmbF5MPZNednC2y/7jf/CUGZ9ivM="),
                ]),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_to_hash_by_wallet() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .get_wallet_transactions_to_hash(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                None,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_to_hash_by_wallet_account() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .get_wallet_transactions_to_hash(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                Some(String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                )),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_create_wallet_transaction() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let payload = CreateWalletTransactionRequestBody {
            TransactionID: String::from("-----BEGIN PGP MESSAGE-----\nVersion: ProtonMail\n\nwV4DV4P+kOmxMSUSAQdASrNn/jIFP6n+AjXwVk6VfU2SUiqGlmG7TkTtZijw\nkBowwibck93WAs73xSsUgbT1BNjRKeYVuZV6hdH+j9DImHBZqrzGmvkR6TNz\n3c9E2t520nEB9VnJbGKMkmsE8hKoL+aIGEvoeO5zAB5sCFKkxF0n/Ij5GkQE\nv7+nj8rTnyGOvkja9koS4lE0waUoSwswGPu/L1JUGLvZVai8Yc13ensyULmD\ngzZMClFfYeDNoXKYzXcXSYsU+FQRyljyB64zD0Z3Tw==\n=cx4x\n-----END PGP MESSAGE-----"),
            HashedTransactionID: String::from("XYgTAERpwkoYogPUWvlfmyaK17q7DTmkwDHdvpptrGc"),
            Label: Some(String::from("xyz")),
            ExchangeRateID: Some(String::from("pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==")),
            TransactionTime: None,
        };

        let res = client
            .create_wallet_transaction(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                payload,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_transaction_label() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .update_wallet_transaction_label(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
                String::from("xyz"),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_transaction_hashed_txid() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .update_wallet_transaction_hashed_txid(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
                String::from("bymboZ1s6GaWwT9kCgrOTOVyzcPAKfmFYUHJCJy9c6U="),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_transaction_external_sender() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client.update_external_wallet_transaction_sender(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
                String::from("-----BEGIN PGP MESSAGE-----\nVersion: ProtonMail\n\nwV4DV4P+kOmxMSUSAQdASrNn/jIFP6n+AjXwVk6VfU2SUiqGlmG7TkTtZijw\nkBowwibck93WAs73xSsUgbT1BNjRKeYVuZV6hdH+j9DImHBZqrzGmvkR6TNz\n3c9E2t520nEB9VnJbGKMkmsE8hKoL+aIGEvoeO5zAB5sCFKkxF0n/Ij5GkQE\nv7+nj8rTnyGOvkja9koS4lE0waUoSwswGPu/L1JUGLvZVai8Yc13ensyULmD\ngzZMClFfYeDNoXKYzXcXSYsU+FQRyljyB64zD0Z3Tw==\n=cx4x\n-----END PGP MESSAGE-----"),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_set_wallet_transaction_private_flag() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .set_wallet_transaction_flag(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
                crate::wallet::WalletTransactionFlag::Private,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_set_wallet_transaction_suspicious_flag() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .set_wallet_transaction_flag(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
                crate::wallet::WalletTransactionFlag::Suspicious,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet_transaction_private_flag() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .delete_wallet_transaction_flag(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
                crate::wallet::WalletTransactionFlag::Private,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet_transaction_suspicious_flag() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .delete_wallet_transaction_flag(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
                crate::wallet::WalletTransactionFlag::Suspicious,
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet_transaction() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .delete_wallet_transactions(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                ),
                String::from(
                    "h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                ),
            )
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn should_disable_show_wallet_recovery() {
        let api_client = common_api_client().await;
        let client = WalletClient::new(api_client);

        let res = client
            .disable_show_wallet_recovery(String::from(
                "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
            ))
            .await;

        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    /// Unit tests with mock
    #[tokio::test]
    async fn test_create_wallet_account_2002() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 2002,
                "Details": { },
                "Error": "Attribute DerivationPath is invalid: The data should be a valid BIP 44, 49, 84 or 86 derivation path.",
            }
        );
        let wallet_id = String::from("test_wallet_id");
        let req_path = format!("{}/wallets/{}/accounts", BASE_WALLET_API_V1, wallet_id);
        let response = ResponseTemplate::new(400).set_body_json(response_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let session = setup_test_connection_arc(mock_server.uri());
        let client = WalletClient::new(session);
        let payload = CreateWalletAccountRequestBody {
            DerivationPath: DerivationPath::from_str("m/44'/1'/0'").unwrap().to_string(),
            Label: String::from("test_label_id"),
            ScriptType: ScriptType::NativeSegwit.into(),
        };
        let res = client.create_wallet_account(wallet_id, payload).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            Error::ErrorCode(_, error) => {
                assert!(error.Code == 2002);
                assert!(error.Error == "Attribute DerivationPath is invalid: The data should be a valid BIP 44, 49, 84 or 86 derivation path.");
            }
            _ => {
                panic!("Expected Ok variant but got Err.")
            }
        }
    }

    #[tokio::test]
    async fn test_create_wallet_account_1000() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "Account": {
                    "ID": "string",
                    "WalletID": "string",
                    "DerivationPath": "m/44'/0'/0'",
                    "Label": "string",
                    "LastUsedIndex": 666,
                    "PoolSize": 12,
                    "Priority": 23,
                    "ScriptType": 1,
                    "Addresses": [],
                    "FiatCurrency": "USD",
                }
            }
        );
        let wallet_id = String::from("test_wallet_id");
        let req_path = format!("{}/wallets/{}/accounts", BASE_WALLET_API_V1, wallet_id);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let client = WalletClient::new(api_client);
        let payload = CreateWalletAccountRequestBody {
            DerivationPath: DerivationPath::from_str("m/44'/1'/0'").unwrap().to_string(),
            Label: String::from("test_label_id"),
            ScriptType: ScriptType::NativeSegwit.into(),
        };
        let res = client.create_wallet_account(wallet_id, payload).await;
        assert!(res.is_ok());
        let wallet_account = res.unwrap();
        assert_eq!(wallet_account.DerivationPath, "m/44'/0'/0'");
        assert_eq!(wallet_account.Label, "string");
        assert_eq!(wallet_account.ScriptType, 1);
        assert_eq!(wallet_account.Priority, 23);
        assert_eq!(wallet_account.WalletID, "string");
        assert_eq!(wallet_account.ID, "string");
        assert_eq!(wallet_account.LastUsedIndex, 666);
        assert_eq!(wallet_account.PoolSize, 12);
    }

    #[tokio::test]
    async fn test_create_wallet_transaction_1000() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "WalletTransaction": {
                    "ID":"h3fiHve6jGce6SiAB14JJpusSHlRZT01jQWI-DK6Cc4aY8w_4qqyL8eNS021UNUJAZmT3XT5XnhQWIW97XYkpw==",
                    "WalletID":"pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                    "WalletAccountID":"lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==",
                    "Label":"xyw=",
                    "TransactionID":"-----BEGIN PGP MESSAGE-----\nVersion: ProtonMail\n\nwV4DV4P+kOmxMSUSAQdASrNn/jIFP6n+AjXwVk6VfU2SUiqGlmG7TkTtZijw\nkBowwibck93WAs73xSsUgbT1BNjRKeYVuZV6hdH+j9DImHBZqrzGmvkR6TNz\n3c9E2t520nEB9VnJbGKMkmsE8hKoL+aIGEvoeO5zAB5sCFKkxF0n/Ij5GkQE\nv7+nj8rTnyGOvkja9koS4lE0waUoSwswGPu/L1JUGLvZVai8Yc13ensyULmD\ngzZMClFfYeDNoXKYzXcXSYsU+FQRyljyB64zD0Z3Tw==\n=cx4x\n-----END PGP MESSAGE-----\n",
                    "TransactionTime":"1714553312",
                    "IsSuspicious": 1,
                    "IsPrivate": 1,
                    "ExchangeRate":{
                        "ID":"pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                        "BitcoinUnit":"BTC",
                        "FiatCurrency":"EUR",
                        "ExchangeRateTime":"1714553312",
                        "ExchangeRate":5334511,
                        "Cents":100,
                    },
                    "HashedTransactionID":"bymboZ1s6GaWwT9kCgrOTOVyzcPAKfmFYUHJCJy9c6U=",
                    "Subject": null,
                    "Body": null,
                    "ToList": null,
                    "Sender": null,
                }
            }
        );
        let wallet_id =
            String::from("pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==");
        let wallet_account_id =
            String::from("lY2ZCYkVNfl_osze70PRoqzg34MQI64mE3-pLc-yMp_6KXthkV1paUsyS276OdNwucz9zKoWKZL_TgtKxOPb0w==");
        let transaction_id = String::from("-----BEGIN PGP MESSAGE-----\nVersion: ProtonMail\n\nwV4DV4P+kOmxMSUSAQdASrNn/jIFP6n+AjXwVk6VfU2SUiqGlmG7TkTtZijw\nkBowwibck93WAs73xSsUgbT1BNjRKeYVuZV6hdH+j9DImHBZqrzGmvkR6TNz\n3c9E2t520nEB9VnJbGKMkmsE8hKoL+aIGEvoeO5zAB5sCFKkxF0n/Ij5GkQE\nv7+nj8rTnyGOvkja9koS4lE0waUoSwswGPu/L1JUGLvZVai8Yc13ensyULmD\ngzZMClFfYeDNoXKYzXcXSYsU+FQRyljyB64zD0Z3Tw==\n=cx4x\n-----END PGP MESSAGE-----\n");

        let req_path = format!(
            "{}/wallets/{}/accounts/{}/transactions",
            BASE_WALLET_API_V1, wallet_id, wallet_account_id
        );
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("POST"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection_arc(mock_server.uri());
        let client = WalletClient::new(api_client);
        let payload = CreateWalletTransactionRequestBody {
            TransactionID: transaction_id,
            HashedTransactionID: String::from("XYgTAERpwkoYogPUWvlfmyaK17q7DTmkwDHdvpptrGc"),
            Label: Some(String::from("xyz")),
            ExchangeRateID: Some(String::from(
                "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
            )),
            TransactionTime: None,
        };

        let res = client
            .create_wallet_transaction(wallet_id, wallet_account_id, payload)
            .await;
        assert!(res.is_ok());
        let wallet_transaction = res.unwrap();
        assert!(wallet_transaction.Label == Some(String::from("xyw=")));
        assert!(wallet_transaction.TransactionID == "-----BEGIN PGP MESSAGE-----\nVersion: ProtonMail\n\nwV4DV4P+kOmxMSUSAQdASrNn/jIFP6n+AjXwVk6VfU2SUiqGlmG7TkTtZijw\nkBowwibck93WAs73xSsUgbT1BNjRKeYVuZV6hdH+j9DImHBZqrzGmvkR6TNz\n3c9E2t520nEB9VnJbGKMkmsE8hKoL+aIGEvoeO5zAB5sCFKkxF0n/Ij5GkQE\nv7+nj8rTnyGOvkja9koS4lE0waUoSwswGPu/L1JUGLvZVai8Yc13ensyULmD\ngzZMClFfYeDNoXKYzXcXSYsU+FQRyljyB64zD0Z3Tw==\n=cx4x\n-----END PGP MESSAGE-----\n");
        assert!(wallet_transaction.TransactionTime == "1714553312");
        assert!(
            wallet_transaction.HashedTransactionID
                == Some(String::from("bymboZ1s6GaWwT9kCgrOTOVyzcPAKfmFYUHJCJy9c6U="))
        );
        assert!(wallet_transaction.IsSuspicious == 1);
        assert!(wallet_transaction.IsPrivate == 1);
        assert!(wallet_transaction.Subject == None);
        assert!(wallet_transaction.Body == None);
        assert!(wallet_transaction.ToList == None);
        assert!(wallet_transaction.Sender == None);
    }
}
