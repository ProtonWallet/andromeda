use bdk::bitcoin::Network as BdkNetwork;
use bdk::bitcoin::ScriptBuf as BdkScriptBuf;

/// A Bitcoin script.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Script(pub BdkScriptBuf);

impl Script {
    pub fn new(raw_output_script: Vec<u8>) -> Self {
        let script: BdkScriptBuf = raw_output_script.into();
        Script(script)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

impl From<BdkScriptBuf> for Script {
    fn from(script: BdkScriptBuf) -> Self {
        Script(script)
    }
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
