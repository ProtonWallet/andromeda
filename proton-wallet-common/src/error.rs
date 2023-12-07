use bdk::{Error as BdkError, KeychainKind};
use miniscript::bitcoin::bip32::Error as Bip32Error;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidSecretKey,
    InvalidDescriptor,
    InvalidDerivationPath,
    InvalidAccountIndex,
    InvalidTxId,
    DerivationError,
    SyncError,
    InvalidData,
    CannotComputeTxFees,
    CannotGetFeeEstimation,

    // BDK Errors
    Generic { msg: String },
    NoRecipients,
    NoUtxosSelected,
    OutputBelowDustLimit { output: u8 },
    InsufficientFunds { needed: u64, available: u64 },
    BnBTotalTriesExceeded,
    BnBNoExactMatch,
    UnknownUtxo,
    TransactionNotFound,
    TransactionConfirmed,
    IrreplaceableTransaction,
    FeeRateTooLow { required: String },
    FeeTooLow { required: u64 },
    FeeRateUnavailable,
    MissingKeyOrigin { key: String },
    Key { error: String },
    ChecksumMismatch,
    SpendingPolicyRequired { keychain_kind: KeychainKind },
    MiniscriptPsbt,
    InvalidPolicyPathError { error: String },
    Signer { error: String },
    InvalidOutpoint { outpoint: String },
    Descriptor { error: String },
    Miniscript { error: String },
    Bip32 { error: String },
    Psbt { error: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "Custom error"
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self)
    }
}

impl Into<Error> for BdkError {
    fn into(self) -> Error {
        match self {
            // Generic error
            BdkError::Generic(str) => Error::Generic { msg: str },
            // Cannot build a tx without recipients
            BdkError::NoRecipients => Error::NoRecipients,
            // `manually_selected_only` option is selected but no utxo has been passed
            BdkError::NoUtxosSelected => Error::NoUtxosSelected,
            // Output created is under the dust limit, 546 satoshis
            BdkError::OutputBelowDustLimit(outputs) => Error::OutputBelowDustLimit {
                output: outputs.try_into().unwrap(),
            },
            // Wallet's UTXO set is not enough to cover recipient's requested plus fee
            BdkError::InsufficientFunds { needed, available } => Error::InsufficientFunds { needed, available },
            // Branch and bound coin selection possible attempts with sufficiently big UTXO set could grow
            // exponentially, thus a limit is set, and when hit, this ErrorKind is thrown
            BdkError::BnBTotalTriesExceeded => Error::BnBTotalTriesExceeded,
            // Branch and bound coin selection tries to avoid needing a change by finding the right inputs for
            // the desired outputs plus fee, if there is not such combination this ErrorKind is thrown
            BdkError::BnBNoExactMatch => Error::BnBNoExactMatch,
            // Happens when trying to spend an UTXO that is not in the internal database
            BdkError::UnknownUtxo => Error::UnknownUtxo,
            // Thrown when a tx is not found in the internal database
            BdkError::TransactionNotFound => Error::TransactionNotFound,
            // Happens when trying to bump a transaction that is already confirmed
            BdkError::TransactionConfirmed => Error::TransactionConfirmed,
            // Trying to replace a tx that has a sequence >= `0xFFFFFFFE`
            BdkError::IrreplaceableTransaction => Error::IrreplaceableTransaction,
            // When bumping a tx the fee rate requested is lower than required
            BdkError::FeeRateTooLow { required } => Error::FeeRateTooLow {
                required: required.as_sat_per_vb().to_string(),
            },
            // When bumping a tx the absolute fee requested is lower than replaced tx absolute fee
            BdkError::FeeTooLow { required } => Error::FeeTooLow { required },
            // Node doesn't have data to estimate a fee rate
            BdkError::FeeRateUnavailable => Error::FeeRateUnavailable,
            // In order to use the [`TxBuilder::add_global_xpubs`] option every extended
            // key in the descriptor must either be a master key itself (having depth = 0) or have an
            // explicit origin provided
            //
            // [`TxBuilder::add_global_xpubs`]: crate::wallet::tx_builder::TxBuilder::add_global_xpubs
            BdkError::MissingKeyOrigin(key) => Error::MissingKeyOrigin { key },
            // ErrorKind while working with [`keys`](crate::keys)
            BdkError::Key(error) => Error::Key {
                error: error.to_string(),
            },
            // Descriptor checksum mismatch
            BdkError::ChecksumMismatch => Error::ChecksumMismatch,
            // Spending policy is not compatible with this [`KeychainKind`](crate::types::KeychainKind)
            BdkError::SpendingPolicyRequired(keychain_kind) => Error::SpendingPolicyRequired { keychain_kind },
            // Error while extracting and manipulating policies
            BdkError::InvalidPolicyPathError(error) => Error::InvalidPolicyPathError {
                error: error.to_string(),
            },
            // Signing error
            BdkError::Signer(error) => Error::Signer {
                error: error.to_string(),
            },
            // Requested outpoint doesn't exist in the tx (vout greater than available outputs)
            BdkError::InvalidOutpoint(outpoint) => Error::InvalidOutpoint {
                outpoint: outpoint.to_string(),
            },
            // Error related to the parsing and usage of descriptors
            BdkError::Descriptor(error) => Error::Descriptor {
                error: error.to_string(),
            },
            // Miniscript error
            BdkError::Miniscript(error) => Error::Miniscript {
                error: error.to_string(),
            },
            // Miniscript PSBT error
            BdkError::MiniscriptPsbt(_) => Error::MiniscriptPsbt,
            // BIP32 error
            BdkError::Bip32(error) => Error::Bip32 {
                error: error.to_string(),
            },
            // Partially signed bitcoin transaction error
            BdkError::Psbt(error) => Error::Psbt {
                error: error.to_string(),
            },
        }
    }
}

impl Into<Error> for Bip32Error {
    fn into(self) -> Error {
        Error::Bip32 {
            error: self.to_string(),
        }
    }
}
