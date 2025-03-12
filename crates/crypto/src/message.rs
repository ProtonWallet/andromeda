use std::str::FromStr;

use bitcoin::{
    hashes::{sha256d, Hash},
    secp256k1::{self, ecdsa::RecoverableSignature, All, Message},
    sign_message::signed_msg_hash,
};

use crate::error::Error;

pub struct BitcoinMessage(Message);

impl BitcoinMessage {
    /// Creates a `BitcoinMessage` from a precomputed hash.
    fn from_hash(hash: sha256d::Hash) -> Result<Self, Error> {
        let message = secp256k1::Message::from_digest(hash.to_byte_array());
        Ok(Self(message))
    }

    /// Signs the message using the given secp256k1 keypair.
    pub fn sign(
        &self,
        secp: &secp256k1::Secp256k1<All>,
        key_pair: &secp256k1::Keypair,
    ) -> Result<RecoverableSignature, Error> {
        let signature = secp.sign_ecdsa_recoverable(&self.0, &key_pair.secret_key());
        secp.verify_ecdsa(&self.0, &signature.to_standard(), &key_pair.public_key())?;
        Ok(signature)
    }
}

impl TryFrom<&str> for BitcoinMessage {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::from_hash(signed_msg_hash(s))
    }
}

impl FromStr for BitcoinMessage {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::sha256d;
    use bitcoin::key::Keypair;
    use bitcoin::secp256k1::{rand::thread_rng, Secp256k1, SecretKey};
    use std::str::FromStr;

    #[test]
    fn test_bitcoin_message_from_string() {
        let msg_str = "Hello, Bitcoin!";
        let bitcoin_msg = BitcoinMessage::from_str(msg_str);
        assert!(bitcoin_msg.is_ok());
    }

    #[test]
    fn test_bitcoin_message_from_hash() {
        let hash = sha256d::Hash::hash(b"Hello, Bitcoin!");
        let bitcoin_msg = BitcoinMessage::from_hash(hash);
        assert!(bitcoin_msg.is_ok());
    }

    #[test]
    fn test_sign_message() {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut thread_rng());
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let message = BitcoinMessage::from_str("Test message").unwrap();
        let signature = message.sign(&secp, &keypair);
        assert!(signature.is_ok());
    }

    #[test]
    fn test_invalid_signing() {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut thread_rng());
        let keypair = Keypair::from_secret_key(&secp, &secret_key);
        let invalid_message = BitcoinMessage::from_hash(sha256d::Hash::from_slice(&[0; 32]).unwrap());
        assert!(invalid_message.is_ok());
        let sign_result = invalid_message.unwrap().sign(&secp, &keypair);
        assert!(sign_result.is_ok());
    }
}
