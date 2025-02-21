use andromeda_common::ScriptType;
use bitcoin::{
    base64::{prelude::BASE64_STANDARD, Engine as _},
    key::Secp256k1,
    secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        All,
    },
    Address,
};

use crate::error::Error;

pub enum SigningType {
    Electrum,
    Bip137,
}

pub struct MessageSignature(bitcoin::sign_message::MessageSignature);

impl MessageSignature {
    /// Creates a new message signature.
    pub fn new(signature: RecoverableSignature, compressed: bool) -> Self {
        MessageSignature(bitcoin::sign_message::MessageSignature { signature, compressed })
    }

    /// Parses a message signature from a base64-encoded string.
    pub fn from_base64(s: &str) -> Result<MessageSignature, Error> {
        let bytes = BASE64_STANDARD.decode(s)?;
        MessageSignature::from_slice(&bytes)
    }

    /// Converts the message signature to a base64-encoded string.
    pub fn to_base64(&self, signing_type: SigningType, script_type: ScriptType) -> Result<String, Error> {
        Ok(BASE64_STANDARD.encode(self.serialize(signing_type, script_type)?))
    }

    /// Parses a message signature from a byte slice.
    fn from_slice(bytes: &[u8]) -> Result<MessageSignature, Error> {
        if bytes.len() != 65 {
            return Err(Error::InvalidSignature);
        }

        let header_value = bytes[0];
        let (script_type, is_key_compressed) = match header_value {
            27..=30 => (ScriptType::Legacy, false),
            31..=34 => (ScriptType::Legacy, true),
            35..=38 => (ScriptType::NestedSegwit, false),
            39..=42 => (ScriptType::NativeSegwit, false),
            _ => return Err(Error::ScriptTypeNotSupported),
        };

        let rec_id = match script_type {
            ScriptType::Legacy => header_value - 27 - (if is_key_compressed { 4 } else { 0 }),
            ScriptType::NestedSegwit => header_value - 35,
            ScriptType::NativeSegwit => header_value - 39,
            ScriptType::Taproot => return Err(Error::ScriptTypeNotSupported),
        };

        let recid = RecoveryId::from_i32(rec_id as i32)?;
        Ok(MessageSignature(bitcoin::sign_message::MessageSignature {
            signature: RecoverableSignature::from_compact(&bytes[1..], recid)?,
            compressed: is_key_compressed,
        }))
    }

    /// Serializes the message signature into a 65-byte array.
    fn serialize(&self, signing_type: SigningType, script_type: ScriptType) -> Result<[u8; 65], Error> {
        let (recid, raw) = self.0.signature.serialize_compact();
        let script_type_check = match signing_type {
            // Electrum uses Legacy format for message signing
            SigningType::Electrum => ScriptType::Legacy,
            SigningType::Bip137 => script_type,
        };
        let header = match script_type_check {
            ScriptType::Legacy => 27 + if self.0.compressed { 4 } else { 0 },
            ScriptType::NestedSegwit => 35,
            ScriptType::NativeSegwit => 39,
            ScriptType::Taproot => return Err(Error::ScriptTypeNotSupported),
        } + recid.to_i32() as u8;
        // Signature output is 65 bytes [1][32][32] = [header][r][s]
        let mut serialized = [0u8; 65];
        serialized[0] = header;
        serialized[1..].copy_from_slice(&raw);
        Ok(serialized)
    }

    /// Verifies the message signature against a given address.
    ///
    /// Notes:
    /// Verifies the message signature by recovering the public key and comparing it directly,
    /// using rust-bitcoin’s built-in functions. The standard approach typically derives the
    /// address from the recovered key and compares it, but we opted for direct key comparison
    /// for simplicity and reliability
    ///
    /// Additionally, some signatures may appear as uncompressed keys but actually match their
    /// compressed counterparts. This logic ensures compatibility by checking both formats.
    pub fn verify(&self, secp_ctx: &Secp256k1<All>, message: &str, address: Address) -> Result<(), Error> {
        let msg_hash = bitcoin::sign_message::signed_msg_hash(message);
        let pub_key_h = self.0.recover_pubkey(&secp_ctx, msg_hash)?;

        // some signature is parsed as uncompressed key. but it matches comparessed key,
        //   this logic for better complatibility
        let is_valid = address.is_related_to_pubkey(&pub_key_h)
            || (!pub_key_h.compressed && {
                let mut pk = pub_key_h.clone();
                pk.compressed = true;
                address.is_related_to_pubkey(&pk)
            });

        if !is_valid {
            return Err(Error::SignatureVerificationFailed);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        error::Error,
        message_signature::{MessageSignature, SigningType},
    };
    use bitcoin::{
        hashes::Hash,
        key::Keypair,
        secp256k1::{self, SecretKey},
        sign_message::signed_msg_hash,
        Address, Network,
    };
    use std::str::FromStr;

    #[tokio::test]
    async fn test_verify_message_1() {
        let secp_ctx = secp256k1::Secp256k1::new();
        let signature = "IAJllD4KbxxpAx2oXj67d+fNHEcyk+45Pp09HoejR5N2DJMe1Mf7CU4bE91vgLIKBbt0+QQr4F/gNqL4UyYquJg=";
        let signature = MessageSignature::from_base64(signature).unwrap();
        let address = Address::from_str("bc1q63wfn3mxm4jegwle9v4ll4hh7ypsyg786s7gl6")
            .unwrap()
            .require_network(Network::Bitcoin)
            .unwrap();
        let res = signature.verify(&secp_ctx, "Hello world", address);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_verify_message_2() {
        // this signature data is pared uncompressed key. but it matches comparessed key
        let secp_ctx = secp256k1::Secp256k1::new();
        let signature_str = "I6ir3uueMk2nza/ZZitIcrI40n79rC/cmu8oQTsqr+DrG/s4q9X7f1ptvZiGnPbT1Vnw3YogpzRkr6hvyAdl4JU=";
        let signature = MessageSignature::from_base64(signature_str).unwrap();
        let address = Address::from_str("3LovK8cXLbRYFZXLdPmnCmHsRk3YFeC5ny")
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();
        let res = signature.verify(&secp_ctx, "Hello world", address);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_verify_message_3() {
        let secp_ctx = secp256k1::Secp256k1::new();
        let signature_str = "HxZkk1LkHaVAo+S+AJQITO4KO3rXfigQVJ9Jbjzo5NqnQZSg8xpH4zM80CVdfOHRdI8RLfmGzrSgqBXnGWnRS1E=";
        let signature = MessageSignature::from_base64(signature_str).unwrap();
        let address = Address::from_str("bc1qv9j2tjwfymmlcveuxcm094m7ajqvw8zar0tgp9")
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();
        let res = signature.verify(&secp_ctx, "hello world aaa!", address);
        assert!(res.is_ok());
    }
    #[tokio::test]
    async fn test_verify_signature_failed() {
        let secp_ctx = secp256k1::Secp256k1::new();
        let signature_str = "HxZkk1LkHaVAo+S+AJQITO4KO3rXfigQVJ9Jbjzo5NqnQZSg8xpH4zM80CIdfOHRdI8RLfmGzrSgqBXnGWnRS1E=";
        let signature = MessageSignature::from_base64(signature_str).unwrap();
        let address = Address::from_str("bc1q63wfn3mxm4jegwle9v4ll4hh7ypsyg786s7gl6")
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();
        let res = signature.verify(&secp_ctx, "Hello world", address);
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_verify_signature_mock_from_github() {
        let secp_ctx = secp256k1::Secp256k1::new();
        let signature_str = "H3I37ur48/fn52ZvWQT+Mj2wXL36gyjfaN5qcgfiVRTJb1eP1li/IacCQspYnUntiRv8r6GDfJYsdiQ5VzlG3As=";
        let signature = MessageSignature::from_base64(signature_str).unwrap();
        let address = Address::from_str("1LsPb3D1o1Z7CzEt1kv5QVxErfqzXxaZXv")
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();
        let res = signature.verify(&secp_ctx, "testtest", address);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_verify_signature_mock_from_github_1() {
        let secp_ctx = secp256k1::Secp256k1::new();
        let signature_str = "GyMn9AdYeZIPWLVCiAblOOG18Qqy4fFaqjg5rjH6QT5tNiUXLS6T2o7iuWkV1gc4DbEWvyi8yJ8FvSkmEs3voWE=";
        let signature = MessageSignature::from_base64(signature_str).unwrap();
        let address = Address::from_str("1GdKjTSg2eMyeVvPV5Nivo6kR8yP2GT7wF")
            .unwrap()
            .require_network(bitcoin::Network::Bitcoin)
            .unwrap();
        let res = signature.verify(
            &secp_ctx,
            "freenode:#bitcoin-otc:b42f7e7ea336db4109df6badc05c6b3ea8bfaa13575b51631c5178a7",
            address,
        );
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_message_signature_from_valid_slice() {
        let secp_ctx = secp256k1::Secp256k1::new();
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        let keypair = Keypair::from_secret_key(&secp_ctx, &secret_key);
        let msg = secp256k1::Message::from_digest(signed_msg_hash("Hello, Bitcoin!").to_byte_array());
        let signature = secp_ctx.sign_ecdsa_recoverable(&msg, &keypair.secret_key());
        let msg_sig = MessageSignature::new(signature, true);

        let signature_1 = msg_sig
            .to_base64(SigningType::Bip137, andromeda_common::ScriptType::NativeSegwit)
            .unwrap();
        assert!(!signature_1.is_empty());
        let signature_2 = msg_sig
            .to_base64(SigningType::Electrum, andromeda_common::ScriptType::Legacy)
            .unwrap();
        assert!(!signature_2.is_empty());
        let signature_3 = msg_sig
            .to_base64(SigningType::Bip137, andromeda_common::ScriptType::NestedSegwit)
            .unwrap();
        assert!(!signature_3.is_empty());
        let signature_4 = msg_sig.to_base64(SigningType::Bip137, andromeda_common::ScriptType::Taproot);
        assert!(signature_4.is_err());
    }

    #[tokio::test]
    async fn test_verify_signature_failed_parse() {
        let signature_str = "GyMnB1h5D1hCBjgKWjg5MT5tNiUXLS7ajmkVBzgKFijInwUpJhJh";
        let signature = MessageSignature::from_base64(signature_str).err().unwrap();
        assert_eq!(signature.to_string(), Error::InvalidSignature.to_string());
    }
}
