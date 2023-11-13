use proton_wallet_common::KeychainKind;
use proton_wallet_common::BdkNetwork;
use proton_wallet_common::WordCount;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
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
            BdkNetwork::Testnet => WasmNetwork::Testnet,
            BdkNetwork::Regtest => WasmNetwork::Regtest,
            BdkNetwork::Signet => WasmNetwork::Signet,
            _ => panic!("Network {} not supported", network),
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
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

#[wasm_bindgen]
pub enum WasmWordCount {
    Words12,
    Words15,
    Words18,
    Words21,
    Words24,
}

impl From<WasmWordCount> for WordCount {
    fn from(value: WasmWordCount) -> Self {
        match value {
            WasmWordCount::Words12 => WordCount::Words12,
            WasmWordCount::Words15 => WordCount::Words15,
            WasmWordCount::Words18 => WordCount::Words18,
            WasmWordCount::Words21 => WordCount::Words21,
            WasmWordCount::Words24 => WordCount::Words24,
        }
    }
}
