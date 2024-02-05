use bdk::descriptor::DescriptorError;
use bdk::keys::bip39::Error as Bip39Error;
use bdk::Error as BdkError;
use bitcoin::bip32::Error as Bip32Error;
use std::fmt::Debug;

#[derive(Debug)]
pub enum Error {
    AccountNotFound,
    // TODO: Maybe we want to segregate BipXX errors (Bip39/Bip32) from other bdk errors?
    BdkError(BdkError),
    Bip32Error(Bip32Error),
    Bip39Error(Option<Bip39Error>),
    CannotComputeTxFees,
    CannotGetFeeEstimation,
    CannotCreateAddressFromScript,
    CannotGetAddressFromScript,
    DerivationError,
    DescriptorError(DescriptorError),
    InvalidAccountIndex,
    InvalidAddress,
    InvalidData,
    InvalidDescriptor,
    InvalidDerivationPath,
    InvalidNetwork,
    InvalidTxId,
    InvalidScriptType,
    InvalidSecretKey,
    InvalidMnemonic,
    LoadError,
    SyncError,
    TransactionNotFound,
}

impl Into<Error> for DescriptorError {
    fn into(self) -> Error {
        Error::DescriptorError(self)
    }
}

impl Into<Error> for BdkError {
    fn into(self) -> Error {
        Error::BdkError(self)
    }
}

impl Into<Error> for Bip32Error {
    fn into(self) -> Error {
        Error::Bip32Error(self)
    }
}

impl Into<Error> for Bip39Error {
    fn into(self) -> Error {
        Error::Bip39Error(Some(self))
    }
}
