use bdk::descriptor::DescriptorError;
use bdk::keys::bip39::Error as Bip39Error;
use bdk::wallet::coin_selection::Error as CoinSelectionError;
use bdk::wallet::error::{BuildFeeBumpError, CreateTxError};
use bdk::wallet::tx_builder::AddUtxoError;
use bdk::wallet::{ChangeSet, NewError, NewOrLoadError};
use bdk::KeychainKind;
use bdk_chain::PersistBackend;
use miniscript::bitcoin::bip32::Error as Bip32Error;
use std::fmt::Debug;

#[derive(Debug)]
pub enum Error<Storage>
where
    Storage: PersistBackend<ChangeSet>,
{
    InvalidAddress,
    InvalidSecretKey,
    InvalidDescriptor,
    InvalidNetwork,
    InvalidDerivationPath,
    InvalidAccountIndex,
    InvalidScriptType,
    InvalidTxId,
    DerivationError,
    SyncError,
    InvalidData,
    CannotComputeTxFees,
    CannotGetFeeEstimation,
    CannotCreateAddressFromScript,
    InvalidMnemonic,
    DescriptorError(DescriptorError),
    LoadError,
    LockError,
    AccountNotFound,

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
    Bip39 { error: Option<Bip39Error> },
    Psbt { error: String },

    ConnectionFailed,

    CreateTxError(CreateTxError<Storage::WriteError>),
    CoinSelectionError(CoinSelectionError),
    BuildFeeBumpError(BuildFeeBumpError),
    AddUtxoError(AddUtxoError),
    NewError(NewError<Storage::WriteError>),
    NewOrLoadError(NewOrLoadError<Storage::WriteError, Storage::LoadError>),
}

impl<Storage> Into<Error<Storage>> for NewOrLoadError<Storage::WriteError, Storage::LoadError>
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::NewOrLoadError(self)
    }
}

impl<Storage> Into<Error<Storage>> for NewError<Storage::WriteError>
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::NewError(self)
    }
}

impl<Storage> Into<Error<Storage>> for CoinSelectionError
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::CoinSelectionError(self)
    }
}

impl<Storage> Into<Error<Storage>> for AddUtxoError
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::AddUtxoError(self)
    }
}

impl<Storage> Into<Error<Storage>> for CreateTxError<Storage::WriteError>
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::CreateTxError(self)
    }
}

impl<Storage> Into<Error<Storage>> for BuildFeeBumpError
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::BuildFeeBumpError(self)
    }
}

impl<Storage> Into<Error<Storage>> for Bip32Error
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::Bip32 {
            error: self.to_string(),
        }
    }
}

impl<Storage> Into<Error<Storage>> for DescriptorError
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::DescriptorError(self)
    }
}

impl<Storage> Into<Error<Storage>> for Bip39Error
where
    Storage: PersistBackend<ChangeSet>,
{
    fn into(self) -> Error<Storage> {
        Error::Bip39 { error: Some(self) }
    }
}
