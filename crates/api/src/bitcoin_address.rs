use std::sync::Arc;

use muon::Request;
use serde::{Deserialize, Serialize};

use crate::{
    core::{ProtonResponseExt, ToProtonRequest},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};

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
    api_client: Arc<ProtonWalletApiClient>,
}

impl BitcoinAddressClient {
    pub fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    pub async fn get_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        only_without_bitcoin_addresses: Option<u8>,
    ) -> Result<Vec<ApiWalletBitcoinAddress>, Error> {
        let request = self
            .api_client
            .build_full_url(
                BASE_WALLET_API_V1,
                format!("wallets/{}/accounts/{}/addresses/bitcoin", wallet_id, wallet_account_id,),
            )
            .to_get_request()
            .param(
                ONLY_WITHOUT_BITCOIN_ADDRESS_KEY,
                only_without_bitcoin_addresses.map(|o| o.to_string()),
            );

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBitcoinAddressesResponseBody>()?;

        Ok(parsed.WalletBitcoinAddresses)
    }

    pub async fn get_bitcoin_address_highest_index(
        &self,
        wallet_id: String,
        wallet_account_id: String,
    ) -> Result<u64, Error> {
        let request = self
            .api_client
            .build_full_url(
                BASE_WALLET_API_V1,
                format!(
                    "wallets/{}/accounts/{}/addresses/bitcoin/index",
                    wallet_id, wallet_account_id,
                ),
            )
            .to_get_request();

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBitcoinAddressHighestIndexResponseBody>()?;
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

        let request = self
            .api_client
            .build_full_url(
                BASE_WALLET_API_V1,
                format!("wallets/{}/accounts/{}/addresses/bitcoin", wallet_id, wallet_account_id,),
            )
            .to_post_request()
            .json_body(payload)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<GetBitcoinAddressesResponseBody>()?;
        Ok(parsed.WalletBitcoinAddresses)
    }

    pub async fn update_bitcoin_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_account_bitcoin_address_id: String,
        bitcoin_address: ApiBitcoinAddressCreationPayload,
    ) -> Result<ApiWalletBitcoinAddress, Error> {
        let request = self
            .api_client
            .build_full_url(
                BASE_WALLET_API_V1,
                format!(
                    "wallets/{}/accounts/{}/addresses/bitcoin/{}",
                    wallet_id, wallet_account_id, wallet_account_bitcoin_address_id
                ),
            )
            .to_put_request()
            .json_body(bitcoin_address)?;

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateBitcoinAddressResponseBody>()?;
        Ok(parsed.WalletBitcoinAddress)
    }
}
