use andromeda_api::bitcoin_address::{ApiBitcoinAddressCreationPayload, ApiWalletBitcoinAddress, BitcoinAddressClient};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::common::error::ErrorExt;

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiWalletBitcoinAddressLookup {
    pub BitcoinAddress: Option<String>,
    pub BitcoinAddressSignature: Option<String>,
}

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiWalletBitcoinAddress {
    pub ID: String,
    pub WalletID: String,
    pub WalletAccountID: String,
    pub Fetched: u8,
    pub Used: u8,
    pub BitcoinAddress: Option<String>,
    pub BitcoinAddressSignature: Option<String>,
    pub BitcoinAddressIndex: Option<u64>,
}

impl From<ApiWalletBitcoinAddress> for WasmApiWalletBitcoinAddress {
    fn from(value: ApiWalletBitcoinAddress) -> Self {
        Self {
            ID: value.ID,
            WalletID: value.WalletID,
            WalletAccountID: value.WalletAccountID,
            Fetched: value.Fetched,
            Used: value.Used,
            BitcoinAddress: value.BitcoinAddress,
            BitcoinAddressSignature: value.BitcoinAddressSignature,
            BitcoinAddressIndex: value.BitcoinAddressIndex,
        }
    }
}

// We need this wrapper because unfortunately, tsify doesn't support
// VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmApiWalletBitcoinAddressData {
    pub Data: WasmApiWalletBitcoinAddress,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiWalletBitcoinAddresses(pub Vec<WasmApiWalletBitcoinAddressData>);

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(non_snake_case)]
pub struct WasmApiBitcoinAddressCreationPayload {
    pub BitcoinAddress: String,
    pub BitcoinAddressSignature: String,
    pub BitcoinAddressIndex: u64,
}

impl From<WasmApiBitcoinAddressCreationPayload> for ApiBitcoinAddressCreationPayload {
    fn from(value: WasmApiBitcoinAddressCreationPayload) -> Self {
        Self {
            BitcoinAddress: value.BitcoinAddress,
            BitcoinAddressSignature: value.BitcoinAddressSignature,
            BitcoinAddressIndex: value.BitcoinAddressIndex,
        }
    }
}

// We need this wrapper because unfortunately, tsify doesn't support
// VectoIntoWasmAbi yet
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
#[allow(non_snake_case)]
pub struct WasmApiBitcoinAddressCreationPayloadData {
    pub Data: WasmApiBitcoinAddressCreationPayload,
}

#[wasm_bindgen(getter_with_clone)]
pub struct WasmApiBitcoinAddressesCreationPayload(pub Vec<WasmApiBitcoinAddressCreationPayloadData>);

#[wasm_bindgen]
impl WasmApiBitcoinAddressesCreationPayload {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn push(&mut self, create_payload: WasmApiBitcoinAddressCreationPayload) {
        self.0
            .push(WasmApiBitcoinAddressCreationPayloadData { Data: create_payload })
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmBitcoinAddressClient(BitcoinAddressClient);

impl From<BitcoinAddressClient> for WasmBitcoinAddressClient {
    fn from(value: BitcoinAddressClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmBitcoinAddressClient {
    #[wasm_bindgen(js_name = "getBitcoinAddresses")]
    pub async fn get_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        only_without_bitcoin_addresses: Option<u8>,
    ) -> Result<WasmApiWalletBitcoinAddresses, js_sys::Error> {
        let addresses = self
            .0
            .get_bitcoin_addresses(wallet_id, wallet_account_id, only_without_bitcoin_addresses)
            .await
            .map(|addresses| {
                addresses
                    .into_iter()
                    .map(|address| WasmApiWalletBitcoinAddressData { Data: address.into() })
                    .collect::<Vec<_>>()
            })
            .map_err(|e| e.to_js_error())?;

        Ok(WasmApiWalletBitcoinAddresses(addresses))
    }

    #[wasm_bindgen(js_name = "getBitcoinAddressHighestIndex")]
    pub async fn get_bitcoin_address_highest_index(
        &self,
        wallet_id: String,
        wallet_account_id: String,
    ) -> Result<u64, js_sys::Error> {
        self.0
            .get_bitcoin_address_highest_index(wallet_id, wallet_account_id)
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "addBitcoinAddress")]
    pub async fn add_bitcoin_addresses(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        bitcoin_addresses: WasmApiBitcoinAddressesCreationPayload,
    ) -> Result<WasmApiWalletBitcoinAddresses, js_sys::Error> {
        let addresses = self
            .0
            .add_bitcoin_addresses(
                wallet_id,
                wallet_account_id,
                bitcoin_addresses
                    .0
                    .into_iter()
                    .map(|addr| addr.Data.into())
                    .collect::<Vec<_>>(),
            )
            .await
            .map(|addresses| {
                addresses
                    .into_iter()
                    .map(|address| WasmApiWalletBitcoinAddressData { Data: address.into() })
                    .collect::<Vec<_>>()
            })
            .map_err(|e| e.to_js_error())?;

        Ok(WasmApiWalletBitcoinAddresses(addresses))
    }

    #[wasm_bindgen(js_name = "updateBitcoinAddress")]
    pub async fn update_bitcoin_address(
        &self,
        wallet_id: String,
        wallet_account_id: String,
        wallet_account_bitcoin_address_id: String,
        bitcoin_address: WasmApiBitcoinAddressCreationPayload,
    ) -> Result<WasmApiWalletBitcoinAddressData, js_sys::Error> {
        let address = self
            .0
            .update_bitcoin_address(
                wallet_id,
                wallet_account_id,
                wallet_account_bitcoin_address_id,
                bitcoin_address.into(),
            )
            .await
            .map_err(|e| e.to_js_error())?;

        Ok(WasmApiWalletBitcoinAddressData { Data: address.into() })
    }
}
