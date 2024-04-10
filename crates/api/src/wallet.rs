use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Request, Response, Session};
use serde::{Deserialize, Serialize};

use super::BASE_WALLET_API_V1;
use crate::{
    error::{Error, ResponseError},
    exchange_rate::ApiExchangeRate,
};

//TODO:: code need to be used. remove all #[allow(dead_code)]

#[derive(Clone)]
pub struct WalletClient {
    session: Arc<RwLock<Session>>,
}

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
    /// Wallet mnemonic encrypted with the WalletKey, in base64 format
    pub Mnemonic: Option<String>,
    // Unique identifier of the mnemonic, using the first 4 bytes of the master public key hash, required if Mnemonic
    // is provided
    pub Fingerprint: Option<String>,
    /// Wallet master public key encrypted with the WalletKey, in base64 format.
    /// Only allows fetching coins owned by wallet, no spending allowed.
    pub PublicKey: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletKey {
    pub WalletID: String,
    pub UserKeyID: String,
    pub WalletKey: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletSettings {
    pub WalletID: String,
    pub HideAccounts: u8,
    pub InvoiceDefaultDescription: Option<String>,
    pub InvoiceExpirationTime: u64,
    pub MaxChannelOpeningFee: u64,
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
    pub DerivationPath: String,
    pub Label: String,
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
pub struct UpdateWalletAccountLabelRequestBody {
    pub Label: String,
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
struct DeleteWalletAccountResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletTransaction {
    pub ID: String,
    pub WalletID: String,
    pub WalletAccountID: Option<String>,
    pub Label: Option<String>,
    pub TransactionID: String,
    pub TransactionTime: String,
    pub ExchangeRate: Option<ApiExchangeRate>,
    pub HashedTransactionID: Option<String>,
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

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletTransactionLabelRequestBody {
    /// encrypted Base64 encoded binary data
    pub Label: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletTransactionLabelResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletTransaction: ApiWalletTransaction,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletTransactionHashedTxidRequestBody {
    /// Hmac(walletKey, txid) and base64 encoded
    pub HashedTransactionID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletTransactionHashedTxidResponseBody {
    pub Code: u16,
    pub WalletTransaction: ApiWalletTransaction,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DeleteWalletTransactionResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
}

impl WalletClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_wallets(&self) -> Result<Vec<ApiWalletData>, Error> {
        let request = ProtonRequest::new(Method::GET, format!("{}/wallets", BASE_WALLET_API_V1));

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetWalletsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Wallets)
    }

    pub async fn create_wallet(&self, payload: CreateWalletRequestBody) -> Result<ApiWalletData, Error> {
        let request = ProtonRequest::new(Method::POST, format!("{}/wallets", BASE_WALLET_API_V1))
            .json_body(payload)
            .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<CreateWalletResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(ApiWalletData {
            Wallet: parsed.Wallet,
            WalletKey: parsed.WalletKey,
            WalletSettings: parsed.WalletSettings,
        })
    }

    pub async fn update_wallet_name(&self, wallet_id: String, name: String) -> Result<ApiWallet, Error> {
        let payload = UpdateWalletNameRequestBody { Name: name };

        let request = ProtonRequest::new(
            Method::PUT,
            format!("{}/wallets/{}/name", BASE_WALLET_API_V1, wallet_id),
        )
        .json_body(payload)
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<UpdateWalletNameResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Wallet)
    }

    pub async fn delete_wallet(&self, wallet_id: String) -> Result<(), Error> {
        let request = ProtonRequest::new(Method::DELETE, format!("{}/wallets/{}", BASE_WALLET_API_V1, wallet_id));

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        println!("response {:?}", response.to_json::<serde_json::Value>());

        Ok(())
    }

    pub async fn get_wallet_accounts(&self, wallet_id: String) -> Result<Vec<ApiWalletAccount>, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/wallets/{}/accounts", BASE_WALLET_API_V1, wallet_id),
        );

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;
        let parsed = response
            .to_json::<GetWalletAccountsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Accounts)
    }

    pub async fn create_wallet_account(
        &self,
        wallet_id: String,
        payload: CreateWalletAccountRequestBody,
    ) -> Result<ApiWalletAccount, Error> {
        let request = ProtonRequest::new(
            Method::POST,
            format!("{}/wallets/{}/accounts", BASE_WALLET_API_V1, wallet_id),
        )
        .json_body(payload)
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        // at this monment, response.status() is alwasy 200. we need to try parse body
        // to get error if there any
        let parsed = response.to_json::<CreateWalletAccountResponseBody>();
        match parsed {
            Ok(res) => Ok(res.Account),
            Err(_) => {
                let parsed_error = response.to_json::<ResponseError>();
                match parsed_error {
                    Ok(res) => Err(Error::ErrorCode(res)),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub async fn update_wallet_account_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        label: String,
    ) -> Result<ApiWalletAccount, Error> {
        let payload = UpdateWalletAccountLabelRequestBody { Label: label };

        let request = ProtonRequest::new(
            Method::PUT,
            format!(
                "{}/wallets/{}/accounts/{}/label",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        )
        .json_body(payload)
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<UpdateWalletAccountResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Account)
    }

    pub async fn add_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        address_id: String,
    ) -> Result<ApiWalletAccount, Error> {
        let payload = AddEmailAddressRequestBody { AddressID: address_id };

        let request = ProtonRequest::new(
            Method::PUT,
            format!(
                "{}/wallets/{}/accounts/{}/addresses/email",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        )
        .json_body(payload)
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<UpdateWalletAccountResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Account)
    }

    pub async fn remove_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        address_id: String,
    ) -> Result<ApiWalletAccount, Error> {
        let request = ProtonRequest::new(
            Method::DELETE,
            format!(
                "{}/wallets/{}/accounts/{}/addresses/email/{}",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id, address_id
            ),
        );

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<UpdateWalletAccountResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.Account)
    }

    pub async fn delete_wallet_account(&self, wallet_id: String, wallet_account_id: String) -> Result<(), Error> {
        let request = ProtonRequest::new(
            Method::DELETE,
            format!(
                "{}/wallets/{}/accounts/{}",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        );

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let _parsed = response
            .to_json::<DeleteWalletAccountResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(())
    }

    pub async fn get_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
        hashed_txids: Option<Vec<String>>,
    ) -> Result<Vec<ApiWalletTransaction>, Error> {
        let url = match wallet_account_id {
            Some(wallet_account_id) => format!(
                "{}/wallets/{}/accounts/{}/transactions",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
            None => format!("{}/wallets/{}/transactions", BASE_WALLET_API_V1, wallet_id),
        };

        println!("{:?}", url);

        let mut request = ProtonRequest::new(Method::GET, url);

        for txid in hashed_txids.unwrap_or(Vec::new()) {
            request = request.param(HASHED_TRANSACTION_ID_KEY, Some(txid));
        }

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        println!("{:?}", response.to_json::<serde_json::Value>());

        let parsed = response
            .to_json::<GetWalletTransactionsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletTransactions)
    }

    pub async fn get_wallet_transactions_to_hash(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
    ) -> Result<Vec<ApiWalletTransaction>, Error> {
        let url = match wallet_account_id {
            Some(wallet_account_id) => format!(
                "{}/wallets/{}/accounts/{}/transactions/to-hash",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
            None => format!("{}/wallets/{}/transactions/to-hash", BASE_WALLET_API_V1, wallet_id),
        };

        let request = ProtonRequest::new(Method::GET, url);

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<GetWalletTransactionsResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletTransactions)
    }

    pub async fn create_wallet_transaction(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        payload: CreateWalletTransactionRequestBody,
    ) -> Result<ApiWalletTransaction, Error> {
        let request = ProtonRequest::new(
            Method::POST,
            format!(
                "{}/wallets/{}/accounts/{}/transactions",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        )
        .json_body(payload)
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<CreateWalletTransactionResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

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

        let request = ProtonRequest::new(
            Method::PUT,
            format!(
                "{}/wallets/{}/accounts/{}/transactions/{}/label",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id, wallet_transaction_id
            ),
        )
        .json_body(payload)
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<UpdateWalletTransactionLabelResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

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

        let request = ProtonRequest::new(
            Method::PUT,
            format!(
                "{}/wallets/{}/accounts/{}/transactions/{}/hash",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id, wallet_transaction_id
            ),
        )
        .json_body(payload)
        .map_err(|_| Error::SerializeError)?;

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let parsed = response
            .to_json::<UpdateWalletTransactionHashedTxidResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(parsed.WalletTransaction)
    }

    pub async fn delete_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
    ) -> Result<(), Error> {
        let request = ProtonRequest::new(
            Method::DELETE,
            format!(
                "{}/wallets/{}/accounts/{}/transactions/{}",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id, wallet_transaction_id
            ),
        );

        let response = self
            .session
            .read()
            .await
            .bind(request)
            .map_err(|e| e.into())?
            .send()
            .await
            .map_err(|e| e.into())?;

        let _parsed = response
            .to_json::<DeleteWalletTransactionResponseBody>()
            .map_err(|_| Error::DeserializeError)?;

        Ok(())
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
    use crate::{error::Error, utils::common_session, utils_test::setup_test_connection, BASE_WALLET_API_V1};

    #[tokio::test]
    #[ignore]
    async fn should_get_wallets() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let blocks = client.get_wallets().await;
        println!("request done: {:?}", blocks);
    }

    #[tokio::test]
    #[ignore]
    async fn should_create_wallet() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let payload = CreateWalletRequestBody {
            Name: String::from("My test wallet"),
            Type: 1,
            HasPassphrase: 0,
            IsImported: 0,
            Mnemonic: Some(String::from("")),
            Fingerprint: Some(String::from("")),
            PublicKey: None,

            UserKeyID: String::from("A2MiMDdmh59RhGQ13iuZ27tc_vEn5GTf-v1LaCRP99q2rkMmPeuMh1QRdtIjR5UwGAowachcaiYYf8Pcf9tOoA=="),
            WalletKey: String::from("Yituc2t2WS9paWRrTEVLTWRmMW15S3c0b3JoZis0aDA4L3d3SDdzbUJBd3BaaWxzakpNV0xHUmtRQ0wxbEJ2SjlTMHV3N2RIUUd2eWtVdm5ySzJLTmVzclBXVEpwRjVCY1hQOWJaU0ROVTFsa1luR3lZQmFoYXRFMzRwdWM1R0VDUDJTYU5wV0h3PT0="),
        };

        let blocks = client.create_wallet(payload).await;
        println!("request done: {:?}", blocks);
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_name() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .update_wallet_name(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from("My updated wallet"),
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let wallet = client
            .delete_wallet(String::from(
                "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
            ))
            .await;

        println!("request done: {:?}", wallet);
    }

    #[tokio::test]
    #[ignore]
    async fn should_create_wallet_account() {
        let session = common_session().await;
        let client = WalletClient::new(session);

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
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_accounts() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .get_wallet_accounts(String::from(
                "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
            ))
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet_account() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .delete_wallet_account(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "_gsDVeX4osuFvPSlszWb-hGvo7d9poBm58MNxvvC2mmG2F1rfM72IqG3hJvGlgMqRHAMyXGgJCI0J8gfukLlXQ==",
                ),
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_account_label() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .update_wallet_account_label(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "Ac3lBksHTrTEFUJ-LYUVg7Cx2xVLwjw_ZWMyVfYUKo7YFgTTWOj7uINQAGkjzM1HiadZfLDM9J6dJ_r3kJQZ5A==",
                ),
                String::from("QW5vdGhlciB0ZXN0IHdhbGxldCBhY2NvdW50IFhZWg=="),
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_by_wallet() {
        let session = common_session().await;
        let client = WalletClient::new(session);

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
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_by_wallet_account() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .get_wallet_transactions(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                Some(String::from(
                    "nt3NEGgRyn4jA0X-pn0W1b5kBGCdvCLAy4lBMMpIrnedkW38TbMus_mM_2bb4UhIn9I3-EU7mPzQG_nc90SPiQ==",
                )),
                None,
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_with_query_param() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .get_wallet_transactions(
                String::from(
                    "DGnqE6AlZV0bFUNlqjHCEAdmucxmuId02-zjI4xnI6FfWI1NwCsR2JofDXhoVSmLzHvTcqwWJg-4e79vycc_nA==",
                ),
                None,
                Some(vec![
                    String::from("fi9YAgAsh59I7tRLIdYzI5T8Mb21qBFkAmCR9ve2QBk="),
                    String::from("ZxIeKb4btZCywLEMmbF5MPZNednC2y/7jf/CUGZ9ivM="),
                ]),
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_to_hash_by_wallet() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .get_wallet_transactions_to_hash(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                None,
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_get_wallet_transactions_to_hash_by_wallet_account() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .get_wallet_transactions_to_hash(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                Some(String::from(
                    "nt3NEGgRyn4jA0X-pn0W1b5kBGCdvCLAy4lBMMpIrnedkW38TbMus_mM_2bb4UhIn9I3-EU7mPzQG_nc90SPiQ==",
                )),
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_create_wallet_transaction() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let payload = CreateWalletTransactionRequestBody {
            TransactionID: String::from("xyz"),
            HashedTransactionID: String::from("xyz"),
            Label: Some(String::from("xyz")),
            ExchangeRateID: None,
            TransactionTime: None,
        };

        let res = client
            .create_wallet_transaction(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "nt3NEGgRyn4jA0X-pn0W1b5kBGCdvCLAy4lBMMpIrnedkW38TbMus_mM_2bb4UhIn9I3-EU7mPzQG_nc90SPiQ==",
                ),
                payload,
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_transaction_label() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .update_wallet_transaction_label(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "nt3NEGgRyn4jA0X-pn0W1b5kBGCdvCLAy4lBMMpIrnedkW38TbMus_mM_2bb4UhIn9I3-EU7mPzQG_nc90SPiQ==",
                ),
                String::from(
                    "l8vWAXHBQmv0u7OVtPbcqMa4iwQaBqowINSQjPrxAr-Da8fVPKUkUcqAq30_BCxj1X0nW70HQRmAa-rIvzmKUA==",
                ),
                String::from("xyz"),
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_update_wallet_transaction_hashed_txid() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .update_wallet_transaction_hashed_txid(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "nt3NEGgRyn4jA0X-pn0W1b5kBGCdvCLAy4lBMMpIrnedkW38TbMus_mM_2bb4UhIn9I3-EU7mPzQG_nc90SPiQ==",
                ),
                String::from(
                    "l8vWAXHBQmv0u7OVtPbcqMa4iwQaBqowINSQjPrxAr-Da8fVPKUkUcqAq30_BCxj1X0nW70HQRmAa-rIvzmKUA==",
                ),
                String::from("xyz"),
            )
            .await;

        println!("request done: {:?}", res);
    }

    #[tokio::test]
    #[ignore]
    async fn should_delete_wallet_transaction() {
        let session = common_session().await;
        let client = WalletClient::new(session);

        let res = client
            .delete_wallet_transactions(
                String::from(
                    "pIJGEYyNFsPEb61otAc47_X8eoSeAfMSokny6dmg3jg2JrcdohiRuWSN2i1rgnkEnZmolVx4Np96IcwxJh1WNw==",
                ),
                String::from(
                    "nt3NEGgRyn4jA0X-pn0W1b5kBGCdvCLAy4lBMMpIrnedkW38TbMus_mM_2bb4UhIn9I3-EU7mPzQG_nc90SPiQ==",
                ),
                String::from(
                    "l8vWAXHBQmv0u7OVtPbcqMa4iwQaBqowINSQjPrxAr-Da8fVPKUkUcqAq30_BCxj1X0nW70HQRmAa-rIvzmKUA==",
                ),
            )
            .await;

        println!("request done: {:?}", res);
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
        let session = setup_test_connection(mock_server.uri());
        let client = WalletClient::new(session);
        let payload = CreateWalletAccountRequestBody {
            DerivationPath: DerivationPath::from_str("m/44'/1'/0'").unwrap().to_string(),
            Label: String::from("test_label_id"),
            ScriptType: ScriptType::NativeSegwit.into(),
        };
        let res = client.create_wallet_account(wallet_id, payload).await;
        assert!(res.is_err());
        match res.unwrap_err() {
            Error::ErrorCode(code) => {
                assert!(code.code == 2002);
                assert!(code.message == "Attribute DerivationPath is invalid: The data should be a valid BIP 44, 49, 84 or 86 derivation path.");
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
                    "ScriptType": 1,
                    "Addresses": [],
                }
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
        let session = setup_test_connection(mock_server.uri());
        let client = WalletClient::new(session);
        let payload = CreateWalletAccountRequestBody {
            DerivationPath: DerivationPath::from_str("m/44'/1'/0'").unwrap().to_string(),
            Label: String::from("test_label_id"),
            ScriptType: ScriptType::NativeSegwit.into(),
        };
        let res = client.create_wallet_account(wallet_id, payload).await;
        assert!(res.is_ok());
        let walle_account = res.unwrap();
        assert!(walle_account.DerivationPath == "m/44'/0'/0'");
        assert!(walle_account.Label == "string");
        assert!(walle_account.ScriptType == 1);
        assert!(walle_account.WalletID == "string");
        assert!(walle_account.ID == "string");
    }
}
