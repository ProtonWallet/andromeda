use regex::Regex;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An error occurred when init regex: \n\t{0}")]
    RegexError(#[from] regex::Error),

    #[error("Failed to extract message: missing message")]
    MissingMessage,

    #[error("Failed to find the signature format header")]
    SignatureHeader,

    #[error("Failed to extract address or signature from signature section")]
    MissingAddressOrSignature,
}

/// Represents a Bitcoin Signed Message (BSM) in the standard format.
/// This struct holds the message, address, and signature extracted from a signed message.
/// https://en.bitcoin.it/wiki/Message_signing
#[derive(Debug, PartialEq)]
pub struct BitcoinSignedMessage {
    message: String,
    address: String,
    signature: String,
}

impl BitcoinSignedMessage {
    /// Parses a Bitcoin Signed Message (BSM) format.
    ///
    /// # Arguments
    ///
    /// * `signed_bsm` - A &str containing the signed message in BSM format.
    ///
    /// # Returns
    ///
    /// * `Result<Self, Error>` - A `BitcoinSignedMessage` if successful, otherwise an error.
    pub fn parse(signed_bsm: &str) -> Result<Self, crate::error::Error> {
        let re_message = Regex::new(r"(?s)-----BEGIN BITCOIN SIGNED MESSAGE-----\s*(.*?)\s*-----BEGIN SIGNATURE-----")
            .map_err(Error::RegexError)?;
        let re_signature = Regex::new(r"(?s)-----BEGIN SIGNATURE-----\s*(.*?)\s*-----END BITCOIN SIGNED MESSAGE-----")
            .map_err(Error::RegexError)?;

        // Extract message
        let message = re_message
            .captures(signed_bsm)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
            .filter(|m| !m.is_empty()) // Ensure message is not empty
            .ok_or(Error::MissingMessage)?;

        // Extract address and signature
        let sig_section = re_signature
            .captures(signed_bsm)
            .and_then(|caps| caps.get(1))
            .map(|s| s.as_str().trim().to_string())
            .ok_or(Error::SignatureHeader)?;

        let mut lines = sig_section.lines();
        let address = lines
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or(Error::MissingAddressOrSignature)?;

        let signature = lines
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .ok_or(Error::MissingAddressOrSignature)?;

        Ok(Self {
            message,
            address,
            signature,
        })
    }

    /// Generates a Bitcoin Signed Message (BSM) armored format from the struct data.
    ///
    /// # Returns
    ///
    /// * `String` - A formatted Bitcoin Signed Message as a string.
    pub fn generate(&self) -> String {
        format!(
            "-----BEGIN BITCOIN SIGNED MESSAGE-----\n{}\n-----BEGIN SIGNATURE-----\n{}\n{}\n-----END BITCOIN SIGNED MESSAGE-----",
            self.message, self.address, self.signature
        )
    }
}

/// Unit tests for BitcoinSignedMessage
#[cfg(test)]
mod tests {

    use crate::{
        bitcoin_signed_message::{BitcoinSignedMessage, Error},
        read_mock_raw_file,
    };

    #[tokio::test]
    async fn test_parse_valid_message() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/bitcoin_signed_message_1")).unwrap();
        let expected = BitcoinSignedMessage {
            message: "This is a test message.".to_string(),
            address: "1KFHE7w8BhaENAswwryaoccDb6qcT6DbYY".to_string(),
            signature: "SGVsbG8gQml0Y29pbg==".to_string(),
        };
        let parsed = BitcoinSignedMessage::parse(&signed_bsm).unwrap();
        assert_eq!(parsed, expected);

        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/bitcoin_signed_message_3")).unwrap();
        let expected = BitcoinSignedMessage {
            message: "Test".to_string(),
            address: "1BqtNgMrDXnCek3cdDVSer4BK7knNTDTSR".to_string(),
            signature: "ILoOBJK9kVKsdUOnJPPoDtrDtRSQw2pyMo+2r5bdUlNkSLDZLqMs8h9mfDm/alZo3DK6rKvTO0xRPrl6DPDpEik="
                .to_string(),
        };
        let parsed = BitcoinSignedMessage::parse(&signed_bsm).unwrap();
        assert_eq!(parsed, expected);
    }

    #[tokio::test]
    async fn test_generate_message() {
        let msg = BitcoinSignedMessage {
            message: "Hello Bitcoin".to_string(),
            address: "1BitcoinEaterAddressDontSend".to_string(),
            signature: "SGVsbG8gQml0Y29pbg==".to_string(),
        };
        let expected_output = String::from_utf8(read_mock_raw_file!("bsm/bitcoin_signed_message_2")).unwrap();
        assert_eq!(msg.generate(), expected_output);
    }

    #[tokio::test]
    async fn test_parse_invalid_message_format() {
        let invalid_bsm = "Invalid Bitcoin Signed Message Format";
        let result = BitcoinSignedMessage::parse(invalid_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::MissingMessage))
        ));
    }

    #[tokio::test]
    async fn test_parse_invalid_base64_signature() {
        // we dont care if base64 is invalid, we just need to extract it
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/invalid_base64_signature")).unwrap();
        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_missing_address() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/missing_address")).unwrap();
        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::MissingAddressOrSignature))
        ));
    }

    #[tokio::test]
    async fn test_parse_missing_message() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/missing_message")).unwrap();
        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::MissingMessage))
        ));
    }

    #[tokio::test]
    async fn test_parse_empty_message() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/empty_message")).unwrap();
        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::MissingMessage))
        ));
    }
    #[tokio::test]
    async fn test_parse_missing_signature() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/missing_signature")).unwrap();

        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::MissingAddressOrSignature))
        ));
    }

    #[tokio::test]
    async fn test_parse_empty_signature() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/empty_signature")).unwrap();

        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::MissingAddressOrSignature))
        ));
    }

    #[tokio::test]
    async fn test_parse_empty_signature_1() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/empty_signature_1")).unwrap();

        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::MissingAddressOrSignature))
        ));
    }

    #[tokio::test]
    async fn test_parse_empty_signature_2() {
        let signed_bsm = String::from_utf8(read_mock_raw_file!("bsm/empty_signature_2")).unwrap();

        let result = BitcoinSignedMessage::parse(&signed_bsm);
        assert!(matches!(
            result,
            Err(crate::error::Error::BSMError(Error::SignatureHeader))
        ));
    }
}
