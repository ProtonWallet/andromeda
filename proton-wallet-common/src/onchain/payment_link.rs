use std::str::FromStr;

use crate::common::{
    bitcoin::{BitcoinUnit, Network},
    error::Error,
    utils::convert_amount,
};
use bdk::{bitcoin::Address, wallet::ChangeSet};
use bdk_chain::PersistBackend;

use super::account::Account;
use core::fmt::Debug;

use urlencoding::{decode, encode};

#[derive(Debug, PartialEq, Clone)]
pub enum PaymentLink {
    BitcoinAddress(Address),
    BitcoinURI {
        address: Address,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    },
    LightningURI {
        uri: String, // TODO when lightning is supported
    },
    UnifiedURI {
        uri: String, // TODO when lightning is supported
    },
}

impl ToString for PaymentLink {
    fn to_string(&self) -> String {
        match self {
            Self::BitcoinAddress(address) => address.to_string(),
            Self::BitcoinURI {
                address,
                amount,
                label,
                message,
            } => {
                let params_str = Self::get_querystring(amount, label, message);

                if params_str.len() > 0 {
                    return format!("bitcoin:{}?{}", address.to_string(), params_str);
                }

                address.to_string()
            }
            Self::LightningURI { uri } => uri.clone(),
            Self::UnifiedURI { uri } => uri.clone(),
        }
    }
}

const AMOUNT_KEY: &str = "amount";
const LABEL_KEY: &str = "label";
const MESSAGE_KEY: &str = "message";

fn get_query_params(query_params: &Vec<(&str, &str)>, key: &str) -> Option<String> {
    query_params
        .into_iter()
        .find(|(param_key, _)| *param_key == key)
        .map(|(_, value)| decode(value).unwrap().to_string())
}

impl PaymentLink {
    fn get_querystring(amount: &Option<u64>, label: &Option<String>, message: &Option<String>) -> String {
        let str_amount = amount.map(|am| convert_amount(am as f64, BitcoinUnit::SAT, BitcoinUnit::BTC).to_string());

        vec![
            (AMOUNT_KEY, str_amount),
            (LABEL_KEY, label.clone()),
            (MESSAGE_KEY, message.clone()),
        ]
        .into_iter()
        .filter_map(move |(key, value)| value.map(|val| format!("{}={}", key, encode(&val))))
        .collect::<Vec<String>>()
        .join("&")
    }

    /// This acts similarly to PaymentLink.to_string except that it also returns a URI for PaymentLink::BitcoinURI with no argument.
    /// This is useful to create a deeplink
    pub fn to_uri(&self) -> String {
        match self {
            Self::BitcoinAddress(_) => self.to_string(),
            Self::BitcoinURI {
                address,
                amount,
                label,
                message,
            } => {
                let params_str = Self::get_querystring(amount, label, message);

                if params_str.len() > 0 {
                    return format!("bitcoin:{}?{}", address.to_string(), params_str);
                }

                format!("bitcoin:{}", address.to_string())
            }
            Self::LightningURI { uri: _ } => self.to_string(),
            Self::UnifiedURI { uri: _ } => self.to_string(),
        }
    }

    fn try_create_address<Storage>(address_str: &str, network: Network) -> Result<Address, Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet> + Clone,
    {
        let address = Address::from_str(address_str)
            .map_err(|_| Error::InvalidAddress)?
            .require_network(network.into())
            .map_err(|_| Error::InvalidNetwork)?;

        Ok(address)
    }

    pub fn try_parse<Storage>(payment_link_str: String, network: Network) -> Result<PaymentLink, Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet> + Clone,
    {
        if payment_link_str.starts_with("lightning") {
            return Ok(PaymentLink::LightningURI { uri: payment_link_str });
        }

        if payment_link_str.starts_with("bitcoin") {
            // Remove protocol prefix
            let splitted = payment_link_str.split("bitcoin:").collect::<Vec<&str>>()[1];

            // Separate query_string, if any from bitcoin address
            let splitted = splitted.split("?").collect::<Vec<&str>>();

            let (address_str, query_params_str) = match splitted.len() {
                0 => Err(Error::InvalidAddress),
                1 => Ok((splitted[0], "")),
                _ => Ok((splitted[0], splitted[1])),
            }?;

            let address = Self::try_create_address(&address_str, network)?;

            let query_params = querystring::querify(query_params_str);

            let amount = get_query_params(&query_params, AMOUNT_KEY)
                .and_then(|amount_str| amount_str.parse::<f64>().ok())
                .map(|amount| convert_amount(amount, BitcoinUnit::BTC, BitcoinUnit::SAT).round() as u64);

            let label = get_query_params(&query_params, LABEL_KEY);
            let message = get_query_params(&query_params, MESSAGE_KEY);

            return Ok(PaymentLink::BitcoinURI {
                address,
                amount,
                label,
                message,
            });
        }

        let address = Self::try_create_address(&payment_link_str, network)?;

        Ok(PaymentLink::BitcoinAddress(address))
    }

    pub fn new_bitcoin_address<Storage>(
        account: &mut Account<Storage>,
        index: Option<u32>,
    ) -> Result<PaymentLink, Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet> + Clone,
    {
        Ok(PaymentLink::BitcoinAddress(account.get_address(index)?.address))
    }

    pub fn new_bitcoin_uri<Storage>(
        account: &mut Account<Storage>,
        index: Option<u32>,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> Result<PaymentLink, Error<Storage>>
    where
        Storage: PersistBackend<ChangeSet> + Clone,
    {
        let address = account.get_address(index)?.address;

        return Ok(PaymentLink::BitcoinURI {
            address,
            amount,
            label,
            message,
        });
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use miniscript::bitcoin::Address;

    use super::super::payment_link::PaymentLink;
    use crate::common::{bitcoin::Network, error::Error};

    #[test]
    fn should_return_only_address() {
        let payment_link = PaymentLink::BitcoinURI {
            address: Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                .unwrap()
                .assume_checked(),
            amount: None,
            label: None,
            message: None,
        };

        assert_eq!(payment_link.to_string(), "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn");
    }

    #[test]
    fn should_return_uri_with_amount() {
        let payment_link = PaymentLink::BitcoinURI {
            address: Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                .unwrap()
                .assume_checked(),
            amount: Some(166727),
            label: None,
            message: None,
        };

        assert_eq!(
            payment_link.to_string(),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00166727"
        );
    }

    #[test]
    fn should_return_uri_with_encoded_label() {
        let payment_link = PaymentLink::BitcoinURI {
            address: Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                .unwrap()
                .assume_checked(),
            amount: None,
            label: Some("Fermi Pasta".to_string()),
            message: None,
        };

        assert_eq!(
            payment_link.to_string(),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?label=Fermi%20Pasta"
        );
    }

    #[test]
    fn should_return_uri_with_encoded_message() {
        let payment_link = PaymentLink::BitcoinURI {
            address: Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                .unwrap()
                .assume_checked(),
            amount: None,
            label: None,
            message: Some("Thank for your donation".to_string()),
        };

        assert_eq!(
            payment_link.to_string(),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?message=Thank%20for%20your%20donation"
        );
    }

    #[test]
    fn should_return_uri_with_all_params() {
        let payment_link = PaymentLink::BitcoinURI {
            address: Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                .unwrap()
                .assume_checked(),
            amount: Some(166727),
            label: Some("Fermi Pasta".to_string()),
            message: Some("Thank for your donation".to_string()),
        };

        assert_eq!(
            payment_link.to_string(),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00166727&label=Fermi%20Pasta&message=Thank%20for%20your%20donation"
        );
    }

    #[test]
    fn should_parse_str_into_bitcoin_address() {
        assert_eq!(
            PaymentLink::try_parse::<()>(
                "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn".to_string(),
                Network::Testnet
            )
            .unwrap(),
            PaymentLink::BitcoinAddress(
                Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                    .unwrap()
                    .assume_checked()
            )
        );
    }

    #[test]
    fn should_return_error_when_parsing_invalid_btc_address() {
        let error = PaymentLink::try_parse::<()>(
            "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn--".to_string(),
            Network::Testnet,
        )
        .err()
        .unwrap();

        assert_eq!(
            true,
            match error {
                Error::InvalidAddress => true,
                _ => false,
            }
        );
    }

    #[test]
    fn should_return_error_when_parsing_btc_address_with_invalid_network() {
        let error = PaymentLink::try_parse::<()>(
            "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn".to_string(),
            Network::Bitcoin,
        )
        .err()
        .unwrap();

        assert_eq!(
            true,
            match error {
                Error::InvalidNetwork => true,
                _ => false,
            }
        );
    }

    #[test]
    fn should_parse_str_into_bitcoin_uri_with_all_fields() {
        assert_eq!(
            PaymentLink::try_parse::<()>(
                "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00192880&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
                Network::Testnet
            )
            .unwrap(),
            PaymentLink::BitcoinURI {
                address: Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                    .unwrap()
                    .assume_checked(),
                amount: Some(192880),
                label: Some("Fermi Pasta".to_string()),
                message: Some("Thanks for your donation".to_string())
            }
        );
    }

    #[test]
    fn should_parse_str_into_bitcoin_uri_with_no_field() {
        assert_eq!(
            PaymentLink::try_parse::<()>(
                "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn".to_string(),
                Network::Testnet
            )
            .unwrap(),
            PaymentLink::BitcoinURI {
                address: Address::from_str("tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn")
                    .unwrap()
                    .assume_checked(),
                amount: None,
                label: None,
                message: None,
            }
        );
    }

    #[test]
    fn should_return_error_when_parsing_bitcoin_uri_with_invalid_btc_address() {
        let error = PaymentLink::try_parse::<()>(
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn--?amount=0.00192880&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
            Network::Testnet
        ) .err().unwrap();

        assert_eq!(
            true,
            match error {
                Error::InvalidAddress => true,
                _ => false,
            }
        );
    }

    #[test]
    fn should_return_error_when_parsing_bitcoin_uri_with_invalid_network() {
        let error =      PaymentLink::try_parse::<()>(
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00192880&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
            Network::Bitcoin
        )
        .err()
        .unwrap();

        assert_eq!(
            true,
            match error {
                Error::InvalidNetwork => true,
                _ => false,
            }
        );
    }
}
