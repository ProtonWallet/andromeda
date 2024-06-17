use std::fmt;

use bitcoin::{
    bip32::{ChildNumber, DerivationPath},
    Network as BdkNetwork,
};
use error::Error;
use serde::{Deserialize, Serialize};

pub const SATOSHI: u64 = 1;
pub const BITCOIN: u64 = 100_000_000 * SATOSHI;
pub const MILLI_BITCOIN: u64 = BITCOIN / 1000;

pub mod error;
pub mod utils;

/// Reimpl of BDK's Network enum to have exhaustive enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Mainnet Bitcoin.
    Bitcoin,
    /// Bitcoin's testnet network.
    Testnet,
    /// Bitcoin's signet network.
    Signet,
    /// Bitcoin's regtest network.
    Regtest,
}

impl ToString for Network {
    fn to_string(&self) -> String {
        match self {
            Network::Bitcoin => String::from("bitcoin"),
            Network::Testnet => String::from("testnet"),
            Network::Signet => String::from("signet"),
            Network::Regtest => String::from("regtest"),
        }
    }
}

impl From<Network> for BdkNetwork {
    fn from(network: Network) -> Self {
        match network {
            Network::Bitcoin => BdkNetwork::Bitcoin,
            Network::Testnet => BdkNetwork::Testnet,
            Network::Signet => BdkNetwork::Signet,
            Network::Regtest => BdkNetwork::Regtest,
        }
    }
}

impl From<BdkNetwork> for Network {
    fn from(network: BdkNetwork) -> Self {
        match network {
            BdkNetwork::Bitcoin => Network::Bitcoin,
            BdkNetwork::Testnet => Network::Testnet,
            BdkNetwork::Signet => Network::Signet,
            BdkNetwork::Regtest => Network::Regtest,
            _ => panic!("Network {} not supported", network),
        }
    }
}

impl TryFrom<String> for Network {
    type Error = Error;

    fn try_from(network: String) -> Result<Network, Error> {
        let str = network.as_str();

        match str {
            "bitcoin" => Ok(Network::Bitcoin),
            "testnet" => Ok(Network::Testnet),
            "signet" => Ok(Network::Signet),
            "regtest" => Ok(Network::Regtest),
            _ => Err(Error::InvalidNetwork(network)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub enum BitcoinUnit {
    /// 100,000,000 sats
    BTC,
    /// 100,000 sats
    MBTC,
    /// 1 sat
    SATS,
}

impl fmt::Display for BitcoinUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BitcoinUnit::BTC => write!(f, "BTC"),
            BitcoinUnit::MBTC => write!(f, "MBTC"),
            BitcoinUnit::SATS => write!(f, "SATS"),
        }
    }
}

pub trait FromParts {
    fn from_parts(purpose: u32, network: Network, account_index: u32) -> Self;
}

impl FromParts for DerivationPath {
    /// Builds a `DerivationPath` from different parts.
    /// Given BIP32: purpose is used as first index, then network to infer
    /// cointype for second index and finally account index for account-level
    /// derivation at third index ```rust
    /// # use std::str::FromStr;
    /// # use bitcoin::bip32::DerivationPath;
    /// # use andromeda_common::{FromParts, Network};
    /// #
    /// let derivation_path = DerivationPath::from_parts(84, Network::Bitcoin,
    /// 0); assert_eq!(derivation_path,
    /// DerivationPath::from_str("m/84'/0'/0'").unwrap()); ```
    fn from_parts(purpose: u32, network: Network, account: u32) -> Self {
        let purpose_level = ChildNumber::from_hardened_idx(purpose).unwrap();

        let network_index = match network {
            Network::Bitcoin => 0,
            _ => 1,
        };
        let cointype_level = ChildNumber::from_hardened_idx(network_index).unwrap();

        let account_level = ChildNumber::from_hardened_idx(account).unwrap();

        DerivationPath::from(vec![purpose_level, cointype_level, account_level])
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
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

impl From<ScriptType> for u8 {
    fn from(val: ScriptType) -> Self {
        match val {
            ScriptType::Legacy => 1u8,
            ScriptType::NestedSegwit => 2u8,
            ScriptType::NativeSegwit => 3u8,
            ScriptType::Taproot => 4u8,
        }
    }
}

impl TryFrom<u8> for ScriptType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ScriptType::Legacy),
            2 => Ok(ScriptType::NestedSegwit),
            3 => Ok(ScriptType::NativeSegwit),
            4 => Ok(ScriptType::Taproot),
            _ => Err(Error::InvalidScriptType(value.to_string())),
        }
    }
}
