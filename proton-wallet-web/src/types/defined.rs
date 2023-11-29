use proton_wallet_common::bitcoin::Network;
use proton_wallet_common::KeychainKind;
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
