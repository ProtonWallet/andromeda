use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{
    request::{Error as ReqError, Method, ProtonRequest, Response},
    session::Session,
};
use serde::{Deserialize, Serialize};

use super::BASE_WALLET_API_V1;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ScriptType {
    /// Legacy scripts : https://bitcoinwiki.org/wiki/pay-to-pubkey-hash
    Legacy = 1,
    /// Nested segwit scrips : https://bitcoinwiki.org/wiki/pay-to-script-hash
    NestedSegwit = 2,
    /// Native segwit scripts : https://bips.dev/173/
    NativeSegwit = 3,
    /// Taproot scripts : https://bips.dev/341/
    Taproot = 4,
}

impl TryFrom<String> for ScriptType {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "legacy" => Ok(ScriptType::Legacy),
            "nested_segwit" => Ok(ScriptType::NestedSegwit),
            "native_segwit" => Ok(ScriptType::NativeSegwit),
            "taproot" => Ok(ScriptType::Taproot),
            _ => Err("InvalidScriptType"),
        }
    }
}

impl Into<u8> for ScriptType {
    fn into(self) -> u8 {
        match self {
            ScriptType::Legacy => 1u8,
            ScriptType::NestedSegwit => 2u8,
            ScriptType::NativeSegwit => 3u8,
            ScriptType::Taproot => 4u8,
        }
    }
}

impl Into<String> for ScriptType {
    fn into(self) -> String {
        match self {
            ScriptType::Legacy => String::from("legacy"),
            ScriptType::NestedSegwit => String::from("nested_segwit"),
            ScriptType::NativeSegwit => String::from("native_segwit"),
            ScriptType::Taproot => String::from("taproot"),
        }
    }
}

pub struct WalletClient {
    session: Arc<RwLock<Session>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Wallet {
    pub ID: String,
    /// Name of the wallet
    pub Name: String,
    /// 0 if the wallet is created with Proton Wallet
    pub IsImported: u8,
    /// Priority of the wallet (0 is main wallet)
    pub Priority: u8,
    /// 1 is onchain, 2 is lightning
    pub Type: u8,
    /// 1 if the wallet has a passphrase. We don't store it but clients need to request on first wallet access.
    pub HasPassphrase: u8,
    /// 1 means disabled
    pub Status: u8,
    /// Wallet mnemonic encrypted with the WalletKey, in base64 format
    pub Mnemonic: Option<String>,
    /// Wallet master public key encrypted with the WalletKey, in base64 format. Only allows fetching coins owned by wallet, no spending allowed.
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
    /// 1 if the wallet has a passphrase. We don't store it but clients need to request on first wallet access.
    pub HasPassphrase: u8,
    /// Encrypted user Id
    pub UserKeyId: String,
    /// Base64 encoded binary data
    pub WalletKey: String,
    /// Wallet mnemonic encrypted with the WalletKey, in base64 format
    pub Mnemonic: Option<String>,
    /// Wallet master public key encrypted with the WalletKey, in base64 format. Only allows fetching coins owned by wallet, no spending allowed.
    pub PublicKey: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct WalletKey {
    pub UserKeyID: String,
    pub WalletKey: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct WalletSettings {
    pub HideAccounts: u8,
    pub InvoiceDefaultDescription: Option<String>,
    pub InvoiceExpirationTime: u64,
    pub MaxChannelOpeningFee: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct WalletData {
    pub Wallet: Wallet,
    pub WalletKey: WalletKey,
    pub WalletSettings: WalletSettings,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct GetWalletsResponseBody {
    pub Code: u16,
    pub Wallets: Vec<WalletData>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct CreateWalletResponseBody {
    pub Code: u16,
    pub Wallet: Wallet,
    pub WalletKey: WalletKey,
    pub WalletSettings: WalletSettings,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct Account {
    pub ID: String,
    pub DerivationPath: String,
    pub Label: String,
    pub ScriptType: u8,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetWalletAccountsResponseBody {
    pub Code: u16,
    pub Accounts: Vec<Account>,
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
    pub Code: u16,
    pub Account: Account,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct UpdateWalletAccountLabelRequestBody {
    pub Label: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateWalletAccountLabelResponseBody {
    pub Code: u16,
    pub Account: Account,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DeleteWalletAccountResponseBody {
    pub Code: u16,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct WalletTransaction {
    pub ID: String,
    pub WalletID: String,
    pub Label: String,
    pub TransactionID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetWalletTransactionsResponseBody {
    pub Code: u16,
    pub WalletTransactions: Vec<WalletTransaction>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct CreateWalletTransactionRequestBody {
    /// encrypted Base64 encoded binary data
    pub Label: String,
    /// encrypted Base64 encoded binary data
    pub TransactionID: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct CreateWalletTransactionResponseBody {
    pub Code: u16,
    pub WalletTransaction: WalletTransaction,
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
    pub Code: u16,
    pub WalletTransaction: WalletTransaction,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct DeleteWalletTransactionResponseBody {
    pub Code: u16,
}

impl WalletClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_wallets(&self) -> Result<Vec<WalletData>, ReqError> {
        let request = ProtonRequest::new(Method::GET, format!("{}/wallets", BASE_WALLET_API_V1));

        let response = self.session.read().await.bind(request)?.send().await?;
        println!("response here: {:?}", response.to_json::<serde_json::Value>()?);
        let parsed = response.to_json::<GetWalletsResponseBody>()?;

        Ok(parsed.Wallets)
    }

    pub async fn create_wallet(&self, payload: CreateWalletRequestBody) -> Result<WalletData, ReqError> {
        let request = ProtonRequest::new(Method::POST, format!("{}/wallets", BASE_WALLET_API_V1))
            .json_body(payload)
            .unwrap();

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        println!("response here: {:?}", response.to_json::<serde_json::Value>().unwrap());
        let parsed = response.to_json::<CreateWalletResponseBody>().unwrap();

        Ok(WalletData {
            Wallet: parsed.Wallet,
            WalletKey: parsed.WalletKey,
            WalletSettings: parsed.WalletSettings,
        })
    }

    pub async fn get_wallet_accounts(&self, wallet_id: String) -> Result<Vec<Account>, ReqError> {
        let request = ProtonRequest::new(
            Method::GET,
            format!("{}/wallets/{}/accounts", BASE_WALLET_API_V1, wallet_id),
        );

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        let parsed = response.to_json::<GetWalletAccountsResponseBody>().unwrap();

        Ok(parsed.Accounts)
    }

    pub async fn create_wallet_accounts(
        &self,
        wallet_id: String,
        payload: CreateWalletAccountRequestBody,
    ) -> Result<Account, ReqError> {
        let request = ProtonRequest::new(
            Method::POST,
            format!("{}/wallets/{}/accounts", BASE_WALLET_API_V1, wallet_id),
        )
        .json_body(payload)
        .unwrap();

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        println!("response here: {:?}", response.to_json::<serde_json::Value>().unwrap());
        let parsed = response.to_json::<CreateWalletAccountResponseBody>().unwrap();

        Ok(parsed.Account)
    }

    pub async fn update_wallet_account_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        label: String,
    ) -> Result<Account, ReqError> {
        let payload = UpdateWalletAccountLabelRequestBody { Label: label };

        let request = ProtonRequest::new(
            Method::PUT,
            format!(
                "{}/wallets/{}/accounts/{}/label",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        )
        .json_body(payload)
        .unwrap();

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        let parsed = response.to_json::<UpdateWalletAccountLabelResponseBody>().unwrap();

        Ok(parsed.Account)
    }

    pub async fn delete_wallet_account(&self, wallet_id: String, wallet_account_id: String) -> Result<(), ReqError> {
        let request = ProtonRequest::new(
            Method::DELETE,
            format!(
                "{}/wallets/{}/accounts/{}",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        );

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        let _parsed = response.to_json::<DeleteWalletAccountResponseBody>().unwrap();

        Ok(())
    }

    pub async fn get_wallet_transactions(&self, wallet_id: String) -> Result<Vec<WalletTransaction>, ReqError> {
        let request = ProtonRequest::new(Method::GET, format!("{}/wallets/{}", BASE_WALLET_API_V1, wallet_id));

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        let parsed = response.to_json::<GetWalletTransactionsResponseBody>().unwrap();

        Ok(parsed.WalletTransactions)
    }

    pub async fn create_wallet_transaction(
        &self,
        wallet_id: String,
        payload: CreateWalletTransactionRequestBody,
    ) -> Result<WalletTransaction, ReqError> {
        let request = ProtonRequest::new(
            Method::POST,
            format!("{}/wallets/{}/transactions", BASE_WALLET_API_V1, wallet_id),
        )
        .json_body(payload)
        .unwrap();

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        let parsed = response.to_json::<CreateWalletTransactionResponseBody>().unwrap();

        Ok(parsed.WalletTransaction)
    }

    pub async fn update_wallet_transaction_label(
        &self,
        wallet_id: String,
        wallet_transaction_id: String,
        label: String,
    ) -> Result<WalletTransaction, ReqError> {
        let payload = UpdateWalletTransactionLabelRequestBody { Label: label };

        let request = ProtonRequest::new(
            Method::PUT,
            format!(
                "{}/wallets/{}/transactions/{}/label",
                BASE_WALLET_API_V1, wallet_id, wallet_transaction_id
            ),
        )
        .json_body(payload)
        .unwrap();

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        let parsed = response.to_json::<UpdateWalletTransactionLabelResponseBody>().unwrap();

        Ok(parsed.WalletTransaction)
    }

    pub async fn delete_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_transaction_id: String,
    ) -> Result<(), ReqError> {
        let request = ProtonRequest::new(
            Method::DELETE,
            format!(
                "{}/wallets/{}/transactions/{}",
                BASE_WALLET_API_V1, wallet_id, wallet_transaction_id
            ),
        );

        let response = self.session.read().await.bind(request).unwrap().send().await.unwrap();
        let _parsed = response.to_json::<DeleteWalletTransactionResponseBody>().unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CreateWalletAccountRequestBody, CreateWalletRequestBody, ScriptType, WalletClient};
    use crate::utils::common_session;
    use bitcoin::bip32::DerivationPath;
    use std::str::FromStr;

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
            PublicKey: None,

            UserKeyId: String::from("A2MiMDdmh59RhGQ13iuZ27tc_vEn5GTf-v1LaCRP99q2rkMmPeuMh1QRdtIjR5UwGAowachcaiYYf8Pcf9tOoA=="),
            WalletKey: String::from("Yituc2t2WS9paWRrTEVLTWRmMW15S3c0b3JoZis0aDA4L3d3SDdzbUJBd3BaaWxzakpNV0xHUmtRQ0wxbEJ2SjlTMHV3N2RIUUd2eWtVdm5ySzJLTmVzclBXVEpwRjVCY1hQOWJaU0ROVTFsa1luR3lZQmFoYXRFMzRwdWM1R0VDUDJTYU5wV0h3PT0="),
        };

        let blocks = client.create_wallet(payload).await;
        println!("request done: {:?}", blocks);
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
            .create_wallet_accounts(
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
}
