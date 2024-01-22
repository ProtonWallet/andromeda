use bdk::bitcoin::Network as BdkNetwork;

use crate::common::error::Error;

pub const SATOSHI: u64 = 1;
pub const BITCOIN: u64 = 100_000_000 * SATOSHI;
pub const MILLI_BITCOIN: u64 = BITCOIN / 1000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BitcoinUnit {
    /// 100,000,000 sats
    BTC,
    /// 100,000 sats
    MBTC,
    /// 1 sat
    SAT,
}

/// Reimpl of BDK's Network enum to have exhaustive enum
#[derive(Debug, Clone, Copy)]
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

impl TryFrom<String> for Network {
    type Error = Error<()>;

    fn try_from(value: String) -> Result<Self, Error<()>> {
        if value == "bitcoin" {
            return Ok(Network::Bitcoin);
        } else if value == "testnet" {
            return Ok(Network::Testnet);
        } else if value == "signet" {
            return Ok(Network::Signet);
        } else if value == "regtest" {
            return Ok(Network::Regtest);
        }

        Err(Error::InvalidNetwork)
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
