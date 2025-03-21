use andromeda_bitcoin::{BdkNetwork, KeychainKind};
use andromeda_common::{BitcoinUnit, Network, ScriptType};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmBitcoinUnit {
    BTC,
    MBTC,
    SATS,
}

impl From<BitcoinUnit> for WasmBitcoinUnit {
    fn from(val: BitcoinUnit) -> Self {
        match val {
            BitcoinUnit::BTC => WasmBitcoinUnit::BTC,
            BitcoinUnit::MBTC => WasmBitcoinUnit::MBTC,
            BitcoinUnit::SATS => WasmBitcoinUnit::SATS,
        }
    }
}

impl From<WasmBitcoinUnit> for BitcoinUnit {
    fn from(val: WasmBitcoinUnit) -> Self {
        match val {
            WasmBitcoinUnit::BTC => BitcoinUnit::BTC,
            WasmBitcoinUnit::MBTC => BitcoinUnit::MBTC,
            WasmBitcoinUnit::SATS => BitcoinUnit::SATS,
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

impl From<KeychainKind> for WasmKeychainKind {
    fn from(val: KeychainKind) -> Self {
        match val {
            KeychainKind::External => WasmKeychainKind::External,
            KeychainKind::Internal => WasmKeychainKind::Internal,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum WasmScriptType {
    Legacy = 1,
    NestedSegwit = 2,
    NativeSegwit = 3,
    Taproot = 4,
}

impl From<ScriptType> for WasmScriptType {
    fn from(value: ScriptType) -> WasmScriptType {
        match value {
            ScriptType::Legacy => WasmScriptType::Legacy,
            ScriptType::NestedSegwit => WasmScriptType::NestedSegwit,
            ScriptType::NativeSegwit => WasmScriptType::NativeSegwit,
            ScriptType::Taproot => WasmScriptType::Taproot,
        }
    }
}

impl From<WasmScriptType> for ScriptType {
    fn from(val: WasmScriptType) -> Self {
        match val {
            WasmScriptType::Legacy => ScriptType::Legacy,
            WasmScriptType::NestedSegwit => ScriptType::NestedSegwit,
            WasmScriptType::NativeSegwit => ScriptType::NativeSegwit,
            WasmScriptType::Taproot => ScriptType::Taproot,
        }
    }
}

impl From<WasmScriptType> for u8 {
    fn from(val: WasmScriptType) -> Self {
        let script_type: ScriptType = val.into();
        script_type.into()
    }
}

pub trait FromBool {
    fn from_bool(b: bool) -> Self;
}

impl FromBool for u8 {
    fn from_bool(value: bool) -> Self {
        match value {
            true => 1,
            false => 0,
        }
    }
}
