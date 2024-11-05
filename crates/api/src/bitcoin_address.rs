use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_WALLET_API_V1,
};
use muon::common::ServiceType;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl ApiClient for BitcoinAddressClient {
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

impl BitcoinAddressClient {
    pub async fn get_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        only_without_bitcoin_addresses: Option<u8>,
    ) -> Result<Vec<ApiWalletBitcoinAddress>, Error> {
        let mut request = self
            .get(format!(
                "wallets/{}/accounts/{}/addresses/bitcoin",
                wallet_id, wallet_account_id
            ))
            .service_type(ServiceType::Normal, true);
        if let Some(only_without_bitcoin_addresses) = only_without_bitcoin_addresses {
            request = request.query((
                ONLY_WITHOUT_BITCOIN_ADDRESS_KEY,
                only_without_bitcoin_addresses.to_string(),
            ));
        }
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
            .get(format!(
                "wallets/{}/accounts/{}/addresses/bitcoin/index",
                wallet_id, wallet_account_id,
            ))
            .service_type(ServiceType::Normal, true);
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
            .post(format!(
                "wallets/{}/accounts/{}/addresses/bitcoin",
                wallet_id, wallet_account_id,
            ))
            .body_json(payload)?
            .service_type(ServiceType::Normal, false);

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
            .put(format!(
                "wallets/{}/accounts/{}/addresses/bitcoin/{}",
                wallet_id, wallet_account_id, wallet_account_bitcoin_address_id
            ))
            .body_json(bitcoin_address)?
            .service_type(ServiceType::Normal, true);

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<UpdateBitcoinAddressResponseBody>()?;
        Ok(parsed.WalletBitcoinAddress)
    }
}
