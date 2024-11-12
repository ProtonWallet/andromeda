use core::fmt::Debug;
use std::fmt::Display;
use std::str::FromStr;

use andromeda_common::{BitcoinUnit, Network};
use bitcoin::Address;
use urlencoding::{decode, encode};

use crate::{error::Error, utils::convert_amount};

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

impl Display for PaymentLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::BitcoinAddress(address) => address.to_string(),
            Self::BitcoinURI {
                address,
                amount,
                label,
                message,
            } => {
                let params_str = Self::get_querystring(amount, label, message);

                if !params_str.is_empty() {
                    return write!(f, "{}", format!("bitcoin:{}?{}", address, params_str));
                }

                address.to_string()
            }
            Self::LightningURI { uri } => uri.clone(),
            Self::UnifiedURI { uri } => uri.clone(),
        };
        write!(f, "{}", str)
    }
}

const AMOUNT_KEY: &str = "amount";
const LABEL_KEY: &str = "label";
const MESSAGE_KEY: &str = "message";

fn get_query_params(query_params: &Vec<(&str, &str)>, key: &str) -> Option<String> {
    query_params
        .iter()
        .find(|(param_key, _)| *param_key == key)
        .map(|(_, value)| decode(value).unwrap().to_string())
}

impl PaymentLink {
    fn get_querystring(amount: &Option<u64>, label: &Option<String>, message: &Option<String>) -> String {
        let str_amount = amount.map(|am| convert_amount(am as f64, BitcoinUnit::SATS, BitcoinUnit::BTC).to_string());

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

    /// This acts similarly to PaymentLink.to_string except that it also returns
    /// a URI for PaymentLink::BitcoinURI with no argument. This is useful
    /// to create a deeplink
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

                if !params_str.is_empty() {
                    return format!("bitcoin:{}?{}", address, params_str);
                }

                format!("bitcoin:{}", address)
            }
            Self::LightningURI { uri: _ } => self.to_string(),
            Self::UnifiedURI { uri: _ } => self.to_string(),
        }
    }

    fn try_create_address(address_str: &str, network: Network) -> Result<Address, Error> {
        let address = Address::from_str(address_str)?.require_network(network.into())?;

        Ok(address)
    }

    pub fn try_parse(payment_link_str: String, network: Network) -> Result<PaymentLink, Error> {
        if payment_link_str.starts_with("lightning") {
            return Ok(PaymentLink::LightningURI { uri: payment_link_str });
        }

        if payment_link_str.starts_with("bitcoin") {
            // Remove protocol prefix
            let splitted = payment_link_str.split("bitcoin:").collect::<Vec<&str>>()[1];

            // Separate query_string, if any from bitcoin address
            let splitted = splitted.split('?').collect::<Vec<&str>>();

            let (address_str, query_params_str) = match splitted.len() {
                0 => Err(Error::InvalidAddress(payment_link_str.to_string())),
                1 => Ok((splitted[0], "")),
                _ => Ok((splitted[0], splitted[1])),
            }?;

            let address = Self::try_create_address(address_str, network)?;

            let query_params = querystring::querify(query_params_str);

            let amount = get_query_params(&query_params, AMOUNT_KEY)
                .and_then(|amount_str| amount_str.parse::<f64>().ok())
                .map(|amount| convert_amount(amount, BitcoinUnit::BTC, BitcoinUnit::SATS).round() as u64);

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

    pub fn new_bitcoin_uri(
        address: Address,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    ) -> PaymentLink {
        PaymentLink::BitcoinURI {
            address,
            amount,
            label,
            message,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use andromeda_common::Network;
    use bitcoin::{address::ParseError, base58::Error as Base58Error};
    use miniscript::bitcoin::Address;

    use super::super::payment_link::PaymentLink;
    use crate::error::Error;

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
            PaymentLink::try_parse(
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
        let error = PaymentLink::try_parse(
            "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn--".to_string(),
            Network::Testnet,
        )
        .err()
        .unwrap();

        assert!(match error {
            Error::BitcoinAddressParse(error) => match error {
                ParseError::Base58(error) => {
                    match error {
                        Base58Error::Decode(error) => error.invalid_base58_character() == 48,
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        });
    }

    #[test]
    fn should_return_error_when_parsing_btc_address_with_invalid_network() {
        let error = PaymentLink::try_parse(
            "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn".to_string(),
            Network::Bitcoin,
        )
        .err()
        .unwrap();

        assert!(match error {
            Error::BitcoinAddressParse(error) => match error {
                ParseError::NetworkValidation(error) =>
                    error.to_string() == "address tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn is not valid on bitcoin",
                _ => false,
            },
            _ => false,
        });
    }

    #[test]
    fn should_parse_str_into_bitcoin_uri_with_all_fields() {
        assert_eq!(
            PaymentLink::try_parse(
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
            PaymentLink::try_parse(
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
        let error = PaymentLink::try_parse(
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn--?amount=0.00192880&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
            Network::Testnet
        ) .err().unwrap();

        assert!(match error {
            Error::BitcoinAddressParse(error) => match error {
                ParseError::Base58(error) => {
                    match error {
                        Base58Error::Decode(error) => error.invalid_base58_character() == 48,
                        _ => false,
                    }
                }
                _ => false,
            },
            _ => false,
        });
    }

    #[test]
    fn should_return_error_when_parsing_bitcoin_uri_with_invalid_network() {
        let error =      PaymentLink::try_parse(
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00192880&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
            Network::Bitcoin
        )
        .err()
        .unwrap();

        println!("errorerrorerror {:?}", error);

        assert!(match error {
            Error::BitcoinAddressParse(error) => match error {
                ParseError::NetworkValidation(error) =>
                    error.to_string() == "address tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn is not valid on bitcoin",
                _ => false,
            },
            _ => false,
        });
    }
}
