use proton_wallet_common::error::Error;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub enum WasmError {
    InvalidSecretKey,
    InvalidDescriptor,
    InvalidDerivationPath,
    InvalidAccountIndex,
    DerivationError,
    SyncError,
}

impl Into<WasmError> for Error {
    fn into(self) -> WasmError {
        match self {
            Error::InvalidSecretKey => WasmError::InvalidSecretKey,
            Error::InvalidDescriptor => WasmError::InvalidDescriptor,
            Error::InvalidDerivationPath => WasmError::InvalidDerivationPath,
            Error::InvalidAccountIndex => WasmError::InvalidAccountIndex,
            Error::DerivationError => WasmError::DerivationError,
            Error::SyncError => WasmError::SyncError,
        }
    }
}
