use std::sync::Arc;

use async_std::sync::RwLock;
use muon::{http::Method, ProtonRequest, Request, Response, Session};
use serde::{Deserialize, Serialize};

use crate::{error::Error, BASE_WALLET_API_V1};

const ONLY_WITHOUT_BITCOIN_ADDRESS_KEY: &str = "OnlyWithoutBitcoinAddresses[]";

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiWalletBitcoinAddress {
    pub ID: String,
    pub WalletID: String,
    pub WalletAccountID: String,
    pub Fetched: u8,
    pub Used: u8,
    pub BitcoinAddress: Option<String>,
    pub BitcoinAddressSignature: Option<String>,
    pub BitcoinAddressIndex: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AddBitcoinAddressesRequestBody {
    pub BitcoinAddresses: Vec<ApiBitcoinAddressCreationPayload>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiBitcoinAddressCreationPayload {
    pub BitcoinAddress: String,
    pub BitcoinAddressSignature: String,
    pub BitcoinAddressIndex: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBitcoinAddressHighestIndexResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub HighestIndex: u64,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct GetBitcoinAddressesResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddresses: Vec<ApiWalletBitcoinAddress>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UpdateBitcoinAddressResponseBody {
    #[allow(dead_code)]
    pub Code: u16,
    pub WalletBitcoinAddress: ApiWalletBitcoinAddress,
}

#[derive(Clone)]
pub struct BitcoinAddressClient {
    session: Arc<RwLock<Session>>,
}

impl BitcoinAddressClient {
    pub fn new(session: Arc<RwLock<Session>>) -> Self {
        Self { session }
    }

    pub async fn get_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        only_without_bitcoin_addresses: Option<u8>,
    ) -> Result<Vec<ApiWalletBitcoinAddress>, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!(
                "{}/wallets/{}/accounts/{}/addresses/bitcoin",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id,
            ),
        )
        .param(
            ONLY_WITHOUT_BITCOIN_ADDRESS_KEY,
            only_without_bitcoin_addresses.map(|o| o.to_string()),
        );

        let response = self.session.read().await.bind(request)?.send().await?;

        let parsed = response.to_json::<GetBitcoinAddressesResponseBody>()?;

        Ok(parsed.WalletBitcoinAddresses)
    }

    pub async fn get_bitcoin_address_highest_index(
        &self,
        wallet_id: String,
        wallet_account_id: String,
    ) -> Result<u64, Error> {
        let request = ProtonRequest::new(
            Method::GET,
            format!(
                "{}/wallets/{}/accounts/{}/addresses/bitcoin/index",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        );

        let response = self.session.read().await.bind(request)?.send().await?;

        let parsed = response.to_json::<GetBitcoinAddressHighestIndexResponseBody>()?;

        Ok(parsed.HighestIndex)
    }

    pub async fn add_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        bitcoin_addresses: Vec<ApiBitcoinAddressCreationPayload>,
    ) -> Result<Vec<ApiWalletBitcoinAddress>, Error> {
        let payload = AddBitcoinAddressesRequestBody {
            BitcoinAddresses: bitcoin_addresses,
        };
        let request = ProtonRequest::new(
            Method::POST,
            format!(
                "{}/wallets/{}/accounts/{}/addresses/bitcoin",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id
            ),
        )
        .json_body(payload)?;

        let response = self.session.read().await.bind(request)?.send().await?;

        let parsed = response.to_json::<GetBitcoinAddressesResponseBody>()?;

        Ok(parsed.WalletBitcoinAddresses)
    }

    pub async fn update_bitcoin_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_account_bitcoin_address_id: String,
        bitcoin_address: ApiBitcoinAddressCreationPayload,
    ) -> Result<ApiWalletBitcoinAddress, Error> {
        let request = ProtonRequest::new(
            Method::PUT,
            format!(
                "{}/wallets/{}/accounts/{}/addresses/bitcoin/{}",
                BASE_WALLET_API_V1, wallet_id, wallet_account_id, wallet_account_bitcoin_address_id
            ),
        )
        .json_body(bitcoin_address)?;

        let response = self.session.read().await.bind(request)?.send().await?;

        let parsed = response.to_json::<UpdateBitcoinAddressResponseBody>()?;

        Ok(parsed.WalletBitcoinAddress)
    }
}
