use core::fmt::Debug;
use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use andromeda_common::{BitcoinUnit, Network};
use bitcoin::Address;
use urlencoding::{decode, encode};

use super::Result;
use crate::{error::Error, utils::convert_amount};

/// Enum representing different types of payment links for Bitcoin and
/// Lightning.
#[derive(Debug, PartialEq, Clone)]
pub enum PaymentLink {
    /// Basic Bitcoin address.
    BitcoinAddress(Address),

    /// BIP-21 compliant Bitcoin URI, which may include additional parameters
    BitcoinURI {
        address: Address,
        amount: Option<u64>,
        label: Option<String>,
        message: Option<String>,
    },
    /// Placeholder for future Lightning URI support.
    LightningURI { uri: String },
    /// Placeholder for Unified URI support.
    UnifiedURI { uri: String },
}

impl Display for PaymentLink {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::BitcoinAddress(address) => address.to_string(),
            Self::BitcoinURI {
                address,
                amount,
                label,
                message,
            } => {
                let params_str = Self::get_query_string(amount, label, message);
                if !params_str.is_empty() {
                    format!("bitcoin:{}?{}", address, params_str)
                } else {
                    address.to_string()
                }
            }
            Self::LightningURI { uri } | Self::UnifiedURI { uri } => uri.clone(),
        };
        write!(f, "{}", str)
    }
}

// Constants for query parameter keys in BIP-21 URIs.
const AMOUNT_KEY: &str = "amount";
const LABEL_KEY: &str = "label";
const MESSAGE_KEY: &str = "message";

impl PaymentLink {
    /// Helper function to generate a query string from optional BIP-21
    /// parameters.
    fn get_query_string(amount: &Option<u64>, label: &Option<String>, message: &Option<String>) -> String {
        let str_amount = amount.map(|am| convert_amount(am as f64, BitcoinUnit::SATS, BitcoinUnit::BTC).to_string());
        vec![
            (AMOUNT_KEY, str_amount),
            (LABEL_KEY, label.clone()),
            (MESSAGE_KEY, message.clone()),
        ]
        .into_iter()
        .filter_map(|(key, value)| value.map(|val| format!("{}={}", key, encode(&val))))
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
                let params_str = Self::get_query_string(amount, label, message);
                if !params_str.is_empty() {
                    format!("bitcoin:{}?{}", address, params_str)
                } else {
                    format!("bitcoin:{}", address)
                }
            }
            Self::LightningURI { .. } | Self::UnifiedURI { .. } => self.to_string(),
        }
    }

    /// Returns the address as a string, regardless of the type of payment link.
    pub fn to_address_string(&self) -> String {
        match self {
            Self::BitcoinAddress(address) => address.to_string(),
            Self::BitcoinURI { address, .. } => address.to_string(),
            _ => self.to_string(),
        }
    }

    /// Attempts to create a Bitcoin `Address` from a string, validating against
    /// a specified network.
    fn try_create_address(address_str: &str, network: Network) -> Result<Address> {
        let address = Address::from_str(address_str)?.require_network(network.into())?;
        Ok(address)
    }

    /// Attempts to parse a `PaymentLink` from a string.
    /// Supports Bitcoin addresses, BIP-21 URIs, and Lightning URIs.
    pub fn try_parse(payment_link_str: String, network: Network) -> Result<PaymentLink> {
        // Check if URI is a Lightning URI.
        if payment_link_str.starts_with("lightning") {
            return Ok(PaymentLink::LightningURI { uri: payment_link_str });
        }

        // Check if URI is a Bitcoin URI.
        if payment_link_str.starts_with("bitcoin") {
            // Remove protocol prefix and extract address and query string.
            let parts: Vec<&str> = payment_link_str.split("bitcoin:").collect();
            let address_part = parts.get(1).ok_or(Error::InvalidAddress(payment_link_str.clone()))?;
            let query_split: Vec<&str> = address_part.split('?').collect();

            let address_str = query_split
                .first()
                .ok_or(Error::InvalidAddress(payment_link_str.clone()))?; //Ignore the test coverage for line
            let query_params_str = query_split.get(1).unwrap_or(&"");

            let address = Self::try_create_address(address_str, network)?;
            let query_params = querystring::querify(query_params_str);

            // Parse optional parameters from query string.
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

        // If no protocol, assume a Bitcoin address.
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

/// Helper function to retrieve a query parameter from a list of `(key, value)` pairs.
fn get_query_params(query_params: &[(&str, &str)], key: &str) -> Option<String> {
    query_params
        .iter()
        .find(|(param_key, _)| *param_key == key)
        .and_then(|(_, value)| decode(value).ok().map(|decoded| decoded.into_owned()))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use andromeda_common::Network;
    use bitcoin::{address::ParseError, base58::Error as Base58Error};
    use miniscript::bitcoin::Address;

    use crate::{error::Error, payment_link::PaymentLink};

    const TEST_ADDRESS: &str = "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn";
    /// Helper function to create a test address
    fn test_address() -> Address {
        Address::from_str(TEST_ADDRESS).unwrap().assume_checked()
    }

    #[test]
    fn payment_link_to_string_with_address_only() {
        let payment_link = PaymentLink::BitcoinURI {
            address: test_address(),
            amount: None,
            label: None,
            message: None,
        };
        assert_eq!(payment_link.to_string(), TEST_ADDRESS);
    }

    #[test]
    fn payment_links_with_all_type() {
        let test_lightning_url = "lightning:lnbc2500n1p0yx2zp2pp5ajh5uz8mm0lprvgfzjch5yrlze7yx9shcnfqhvx9y0wnn3cd5srqdqqcqzysxqzfvsp5jc7uzwksah3t5kc04z3dh0g6aelw8p4x9n4vj4k5r6jjjufryxl2rq9qyyssqf7lnsx3wn9asjzj4u5q7uzg9xv7ss4srrtygwjt0hfzd9jvkhxygxmpds0p5ezyf34ynzzc3afddzfdgsak7awwtlcpczy7q2".to_string();
        let result = PaymentLink::try_parse(test_lightning_url.clone(), Network::Testnet).unwrap();
        let uri = result.to_uri();
        assert!(uri == result.to_string());
        assert!(uri == test_lightning_url);

        let payment_link = PaymentLink::try_parse(TEST_ADDRESS.to_string(), Network::Testnet).unwrap();
        let uri = payment_link.to_uri();
        assert!(uri == *TEST_ADDRESS);
        let payment_link = PaymentLink::BitcoinURI {
            address: test_address(),
            amount: None,
            label: None,
            message: None,
        };
        assert_eq!(
            payment_link.to_uri(),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn"
        );
        assert_eq!(payment_link.to_string(), "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn");
    }

    #[test]
    fn to_bitcoin_address_string_with_all_type() {
        let test_lightning_url = "lightning:lnbc2500n1p0yx2zp2pp5ajh5uz8mm0lprvgfzjch5yrlze7yx9shcnfqhvx9y0wnn3cd5srqdqqcqzysxqzfvsp5jc7uzwksah3t5kc04z3dh0g6aelw8p4x9n4vj4k5r6jjjufryxl2rq9qyyssqf7lnsx3wn9asjzj4u5q7uzg9xv7ss4srrtygwjt0hfzd9jvkhxygxmpds0p5ezyf34ynzzc3afddzfdgsak7awwtlcpczy7q2".to_string();
        let result = PaymentLink::try_parse(test_lightning_url.clone(), Network::Testnet).unwrap();
        let bitcoin_address = result.to_address_string();
        assert!(bitcoin_address == test_lightning_url);

        let payment_link = PaymentLink::try_parse(TEST_ADDRESS.to_string(), Network::Testnet).unwrap();
        let bitcoin_address = payment_link.to_address_string();
        assert!(bitcoin_address == *TEST_ADDRESS);

        let payment_link = PaymentLink::BitcoinURI {
            address: test_address(),
            amount: Some(166727),
            label: Some("label tests".to_string()),
            message: Some("Thank for your donation".to_string()),
        };
        let bitcoin_address = payment_link.to_address_string();
        assert!(bitcoin_address == *TEST_ADDRESS);
    }

    #[test]
    fn test_invalid_address_parse_error() {
        let invalid_address = "bitcoin:invalid_address";
        let result = PaymentLink::try_parse(invalid_address.to_string(), Network::Testnet);
        assert!(matches!(result, Err(Error::BitcoinAddressParse(_))));

        let empty_address_uri = "bitcoin:";
        let result = PaymentLink::try_parse(empty_address_uri.to_string(), Network::Testnet);
        assert!(matches!(result, Err(Error::BitcoinAddressParse(_))));

        let invalid_uri = "bitcoin:invalid_address?amount=0.001&label=Donation";
        let result = PaymentLink::try_parse(invalid_uri.to_string(), Network::Testnet);
        assert!(matches!(result, Err(Error::BitcoinAddressParse(_))));
    }

    #[test]
    fn payment_link_to_string_with_amount() {
        let payment_link = PaymentLink::BitcoinURI {
            address: test_address(),
            amount: Some(166727),
            label: None,
            message: None,
        };
        assert_eq!(payment_link.to_uri(), payment_link.to_string());
        assert_eq!(
            payment_link.to_uri(),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00166727"
        );
    }

    #[test]
    fn payment_link_to_string_with_label_only() {
        let payment_link = PaymentLink::BitcoinURI {
            address: test_address(),
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
    fn payment_link_to_string_with_all_parameters() {
        let payment_link = PaymentLink::BitcoinURI {
            address: test_address(),
            amount: Some(192880),
            label: Some("Donation".to_string()),
            message: Some("Thanks for your support!".to_string()),
        };
        assert_eq!(
            payment_link.to_string(),
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.0019288&label=Donation&message=Thanks%20for%20your%20support%21"
        );
    }

    #[test]
    fn payment_link_to_string_with_encoded_message() {
        let payment_link = PaymentLink::BitcoinURI {
            address: test_address(),
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
    fn parse_valid_bitcoin_address_into_payment_link() {
        let result = PaymentLink::try_parse(TEST_ADDRESS.to_string(), Network::Testnet);
        assert!(result.is_ok());
        if let Ok(PaymentLink::BitcoinAddress(address)) = result {
            assert_eq!(address.to_string(), TEST_ADDRESS);
        } else {
            panic!("Expected BitcoinAddress variant");
        }
    }

    #[test]
    fn parse_invalid_bitcoin_address_returns_error() {
        let result = PaymentLink::try_parse("invalid_address".to_string(), Network::Testnet);
        assert!(result.is_err());
    }

    #[test]
    fn parse_valid_bitcoin_address_with_invalid_network_returns_error() {
        let result = PaymentLink::try_parse(TEST_ADDRESS.to_string(), Network::Bitcoin);
        assert!(result.is_err());
    }

    #[test]
    fn parse_invalid_btc_address_returns_specific_error() {
        let error = PaymentLink::try_parse(
            "tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn--".to_string(),
            Network::Testnet,
        )
        .err()
        .unwrap();
        assert!(matches!(
            error,
            Error::BitcoinAddressParse(ParseError::Base58(Base58Error::Decode(_)))
        ));
    }

    #[test]
    fn parse_btc_address_with_invalid_network_returns_specific_error() {
        let error = PaymentLink::try_parse(TEST_ADDRESS.to_string(), Network::Bitcoin)
            .err()
            .unwrap();
        assert!(matches!(
            error,
            Error::BitcoinAddressParse(ParseError::NetworkValidation(_))
        ));
    }

    #[test]
    fn parse_bitcoin_uri_with_all_fields() {
        assert_eq!(
            PaymentLink::try_parse(
                "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.00192880&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
                Network::Testnet
            )
            .unwrap(),
            PaymentLink::BitcoinURI {
                address: test_address(),
                amount: Some(192880),
                label: Some("Fermi Pasta".to_string()),
                message: Some("Thanks for your donation".to_string())
            }
        );
    }

    #[test]
    fn parse_bitcoin_uri_with_no_optional_fields() {
        assert_eq!(
            PaymentLink::try_parse(TEST_ADDRESS.to_string(), Network::Testnet).unwrap(),
            PaymentLink::BitcoinAddress(test_address())
        );
    }

    #[test]
    fn parse_str_into_bitcoin_uri_with_no_field() {
        assert_eq!(
            PaymentLink::try_parse(
                "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn".to_string(),
                Network::Testnet
            )
            .unwrap(),
            PaymentLink::BitcoinURI {
                address: test_address(),
                amount: None,
                label: None,
                message: None,
            }
        );
    }

    #[test]
    fn test_return_error_when_parsing_bitcoin_uri_with_invalid_btc_address() {
        let error = PaymentLink::try_parse(
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn--?amount=0.0019288&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
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
    fn test_return_error_when_parsing_bitcoin_uri_with_invalid_network() {
        let error = PaymentLink::try_parse(
            "bitcoin:tb1qnmsyczn68t628m4uct5nqgjr7vf3w6mc0lvkfn?amount=0.0019288&label=Fermi%20Pasta&message=Thanks%20for%20your%20donation".to_string(),
            Network::Bitcoin
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
}
