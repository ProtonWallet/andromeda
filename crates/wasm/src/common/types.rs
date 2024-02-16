use andromeda_bitcoin::{bitcoin::Network, BdkNetwork, KeychainKind};
use andromeda_common::BitcoinUnit;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::common::error::WasmError;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmBitcoinUnit {
    BTC,
    MBTC,
    SAT,
}

impl Into<WasmBitcoinUnit> for BitcoinUnit {
    fn into(self) -> WasmBitcoinUnit {
        match self {
            BitcoinUnit::BTC => WasmBitcoinUnit::BTC,
            BitcoinUnit::MBTC => WasmBitcoinUnit::MBTC,
            BitcoinUnit::SAT => WasmBitcoinUnit::SAT,
        }
    }
}

impl Into<BitcoinUnit> for WasmBitcoinUnit {
    fn into(self) -> BitcoinUnit {
        match self {
            WasmBitcoinUnit::BTC => BitcoinUnit::BTC,
            WasmBitcoinUnit::MBTC => BitcoinUnit::MBTC,
            WasmBitcoinUnit::SAT => BitcoinUnit::SAT,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy)]
pub enum WasmNetwork {
    /// Mainnet Bitcoin.
    Bitcoin,
    /// Bitcoin's testnet network.
    Testnet,
    /// Bitcoin's signet network.
    Signet,
    /// Bitcoin's regtest network.
    Regtest,
}

impl TryFrom<u8> for WasmNetwork {
    type Error = WasmError;

    fn try_from(network: u8) -> Result<WasmNetwork, WasmError> {
        match network {
            0 => Ok(WasmNetwork::Bitcoin),
            1 => Ok(WasmNetwork::Testnet),
            _ => Err(WasmError::InvalidNetwork),
        }
    }
}

impl From<WasmNetwork> for Network {
    fn from(network: WasmNetwork) -> Self {
        match network {
            WasmNetwork::Bitcoin => Network::Bitcoin,
            WasmNetwork::Testnet => Network::Testnet,
            WasmNetwork::Signet => Network::Signet,
            WasmNetwork::Regtest => Network::Regtest,
        }
    }
}

impl From<Network> for WasmNetwork {
    fn from(network: Network) -> Self {
        match network {
            Network::Bitcoin => WasmNetwork::Bitcoin,
            Network::Testnet => WasmNetwork::Testnet,
            Network::Regtest => WasmNetwork::Regtest,
            Network::Signet => WasmNetwork::Signet,
        }
    }
}

impl From<WasmNetwork> for BdkNetwork {
    fn from(network: WasmNetwork) -> Self {
        match network {
            WasmNetwork::Bitcoin => BdkNetwork::Bitcoin,
            WasmNetwork::Testnet => BdkNetwork::Testnet,
            WasmNetwork::Signet => BdkNetwork::Signet,
            WasmNetwork::Regtest => BdkNetwork::Regtest,
        }
    }
}

impl From<BdkNetwork> for WasmNetwork {
    fn from(network: BdkNetwork) -> Self {
        match network {
            BdkNetwork::Bitcoin => WasmNetwork::Bitcoin,
            BdkNetwork::Regtest => WasmNetwork::Regtest,
            BdkNetwork::Signet => WasmNetwork::Signet,
            _ => WasmNetwork::Testnet, // default to testnet, might need to change that
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Serialize, Deserialize)]
pub enum WasmKeychainKind {
    /// External keychain, used for deriving recipient addresses.
    External,
    /// Internal keychain, used for deriving change addresses.
    Internal,
}

impl From<WasmKeychainKind> for KeychainKind {
    fn from(value: WasmKeychainKind) -> Self {
        match value {
            WasmKeychainKind::External => KeychainKind::External,
            WasmKeychainKind::Internal => KeychainKind::Internal,
        }
    }
}

impl Into<WasmKeychainKind> for KeychainKind {
    fn into(self) -> WasmKeychainKind {
        match self {
            KeychainKind::External => WasmKeychainKind::External,
            KeychainKind::Internal => WasmKeychainKind::Internal,
        }
    }
}
