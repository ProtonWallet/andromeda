use andromeda_bitcoin::error::Error;
use core::fmt::Debug;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum WasmError {
    AccountNotFound,
    BdkError,
    Bip32Error,
    Bip39Error,
    CannotComputeTxFees,
    CannotGetFeeEstimation,
    CannotCreateAddressFromScript,
    CannotGetAddressFromScript,
    CannotSignPsbt,
    DerivationError,
    DescriptorError,
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
    LockError,
    OutpointParsingError,
    SyncError,
    TransactionNotFound,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Debug)]
pub struct DetailledWasmError {
    pub kind: WasmError,
    pub details: JsValue,
}

impl Into<DetailledWasmError> for WasmError {
    fn into(self) -> DetailledWasmError {
        DetailledWasmError {
            kind: self,
            details: JsValue::null(),
        }
    }
}

impl Into<WasmError> for Error {
    fn into(self) -> WasmError {
        match self {
            Error::AccountNotFound => WasmError::AccountNotFound,
            Error::BdkError(_) => WasmError::BdkError,
            Error::Bip32Error(_) => WasmError::Bip32Error,
            Error::Bip39Error(_) => WasmError::Bip39Error,
            Error::CannotComputeTxFees => WasmError::CannotComputeTxFees,
            Error::CannotGetFeeEstimation => WasmError::CannotGetFeeEstimation,
            Error::CannotCreateAddressFromScript => WasmError::CannotCreateAddressFromScript,
            Error::CannotGetAddressFromScript => WasmError::CannotGetAddressFromScript,
            Error::DerivationError => WasmError::DerivationError,
            Error::DescriptorError(_) => WasmError::DescriptorError,
            Error::InvalidAccountIndex => WasmError::InvalidAccountIndex,
            Error::InvalidAddress => WasmError::InvalidAddress,
            Error::InvalidData => WasmError::InvalidData,
            Error::InvalidDescriptor => WasmError::InvalidDescriptor,
            Error::InvalidDerivationPath => WasmError::InvalidDerivationPath,
            Error::InvalidNetwork => WasmError::InvalidNetwork,
            Error::InvalidTxId => WasmError::InvalidTxId,
            Error::InvalidScriptType => WasmError::InvalidScriptType,
            Error::InvalidSecretKey => WasmError::InvalidSecretKey,
            Error::InvalidMnemonic => WasmError::InvalidMnemonic,
            Error::LoadError => WasmError::LoadError,
            Error::LockError => WasmError::LockError,
            Error::SyncError => WasmError::SyncError,
            Error::TransactionNotFound => WasmError::TransactionNotFound,
        }
    }
}

impl Into<DetailledWasmError> for Error {
    fn into(self) -> DetailledWasmError {
        let wasm_error: DetailledWasmError = self.into();
        wasm_error.into()
    }
}
