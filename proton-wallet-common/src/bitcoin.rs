use bdk::bitcoin::Network as BdkNetwork;

pub const SATOSHI: u64 = 1;
pub const BITCOIN: u64 = 100_000_000 * SATOSHI;
pub const MILLI_BITCOIN: u64 = BITCOIN / 1000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BitcoinUnit {
    BTC,
    MBTC,
    SAT,
}

/**
 * We reimplement Network enum to have exhaustive enum
 */
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
