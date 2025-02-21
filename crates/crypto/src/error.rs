use std::fmt::Debug;

use bitcoin::sign_message::MessageSignatureError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No signer found")]
    NoSignerFound,
    #[error("Script type not supported")]
    ScriptTypeNotSupported,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid network")]
    InvalidNetwork,
    #[error("Signature header value not set")]
    SignatureHeaderValueNotSet,
    #[error("Decode error: \n\t{0}")]
    Decode(#[from] bitcoin::base64::DecodeError),
    #[error("Crypto secp256k1 error: \n\t{0:?}")]
    Secp256k1(bitcoin::secp256k1::Error),
    #[error("Signature verification failed")]
    SignatureVerificationFailed,
    #[error("Message signature error occurred: \n\t{0}")]
    MessageSignature(MessageSignatureError),
}

// Implement std::convert::From so you can do `?` on secp256k1::Error
impl From<bitcoin::secp256k1::Error> for Error {
    fn from(err: bitcoin::secp256k1::Error) -> Self {
        Error::Secp256k1(err)
    }
}

impl From<MessageSignatureError> for Error {
    fn from(err: MessageSignatureError) -> Self {
        Error::MessageSignature(err)
    }
}
