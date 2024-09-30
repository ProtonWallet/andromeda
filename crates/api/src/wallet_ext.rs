use async_trait::async_trait;

use crate::{
    error::Error,
    settings::FiatCurrencySymbol,
    wallet::{
        ApiEmailAddress, ApiWallet, ApiWalletAccount, ApiWalletData, ApiWalletSettings, ApiWalletTransaction,
        CreateWalletAccountRequestBody, CreateWalletRequestBody, CreateWalletTransactionRequestBody,
        WalletMigrateRequestBody, WalletTransactionFlag,
    },
};

/// Wallet routes trait. This trait defines the methods that the wallet client
/// should implement. also trait could be used to mock the wallet client in
/// tests.
/// Note: when adding new methods to this trait, make sure to implement them in
///  crates/api/src/tests/wallet_mock.rs
///  crates/api/src/wallet.rs
#[async_trait]
pub trait WalletClientExt {
    async fn get_wallets(&self) -> Result<Vec<ApiWalletData>, Error>;

    async fn create_wallet(&self, payload: CreateWalletRequestBody) -> Result<ApiWalletData, Error>;

    async fn migrate(&self, wallet_id: String, payload: WalletMigrateRequestBody) -> Result<(), Error>;

    async fn update_wallet_name(&self, wallet_id: String, name: String) -> Result<ApiWallet, Error>;

    async fn delete_wallet(&self, wallet_id: String) -> Result<(), Error>;

    async fn get_wallet_accounts(&self, wallet_id: String) -> Result<Vec<ApiWalletAccount>, Error>;

    async fn get_wallet_account_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
    ) -> Result<Vec<ApiEmailAddress>, Error>;

    async fn create_wallet_account(
        &self,
        wallet_id: String,
        payload: CreateWalletAccountRequestBody,
    ) -> Result<ApiWalletAccount, Error>;

    async fn update_wallet_account_fiat_currency(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        fiat_currency_symbol: FiatCurrencySymbol,
    ) -> Result<ApiWalletAccount, Error>;

    async fn update_wallet_account_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        label: String,
    ) -> Result<ApiWalletAccount, Error>;

    async fn update_wallet_accounts_order(
        &self,
        wallet_id: String,
        wallet_account_ids: Vec<String>,
    ) -> Result<Vec<ApiWalletAccount>, Error>;

    async fn add_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        address_id: String,
    ) -> Result<ApiWalletAccount, Error>;

    async fn update_wallet_account_last_used_index(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        last_used_index: u32,
    ) -> Result<ApiWalletAccount, Error>;

    async fn remove_email_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        address_id: String,
    ) -> Result<ApiWalletAccount, Error>;

    async fn delete_wallet_account(&self, wallet_id: String, wallet_account_id: String) -> Result<(), Error>;

    async fn get_wallet_transactions(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
        hashed_txids: Option<Vec<String>>,
    ) -> Result<Vec<ApiWalletTransaction>, Error>;

    async fn get_wallet_transactions_to_hash(
        &self,
        wallet_id: String,
        wallet_account_id: Option<String>,
    ) -> Result<Vec<ApiWalletTransaction>, Error>;

    async fn create_wallet_transaction(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        payload: CreateWalletTransactionRequestBody,
    ) -> Result<ApiWalletTransaction, Error>;

    async fn update_wallet_transaction_label(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        label: String,
    ) -> Result<ApiWalletTransaction, Error>;

    async fn update_wallet_transaction_hashed_txid(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        hash_txid: String,
    ) -> Result<ApiWalletTransaction, Error>;

    async fn update_external_wallet_transaction_sender(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        sender: String,
    ) -> Result<ApiWalletTransaction, Error>;

    async fn set_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WalletTransactionFlag,
    ) -> Result<ApiWalletTransaction, Error>;

    async fn delete_wallet_transaction_flag(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
        flag: WalletTransactionFlag,
    ) -> Result<ApiWalletTransaction, Error>;

    async fn delete_wallet_transaction(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_transaction_id: String,
    ) -> Result<(), Error>;

    async fn disable_show_wallet_recovery(&self, wallet_id: String) -> Result<ApiWalletSettings, Error>;
}

#[cfg(test)]
#[cfg(feature = "mocking")]
mod tests {
    use crate::{
        tests::wallet_mock::mock_utils::MockWalletClient, wallet::WalletMigrateRequestBody, wallet_ext::WalletClientExt,
    };

    #[tokio::test]
    async fn test_trait_mocks() {
        let mut mock_wallet_client = MockWalletClient::new();
        let wallet_id = "test_wallet_id".to_string();
        mock_wallet_client
            .expect_migrate()
            .withf(move |id, _| id == &wallet_id)
            .return_once(|_, _| Ok(()));

        let payload = WalletMigrateRequestBody::default();
        let result = mock_wallet_client.migrate("test_wallet_id".to_string(), payload).await;
        assert!(result.is_ok());
    }
}
