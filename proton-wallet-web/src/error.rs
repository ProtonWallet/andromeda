use proton_wallet_common::{error::Error, KeychainKind};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum WasmError {
    InvalidSecretKey,
    InvalidDescriptor,
    InvalidDerivationPath,
    InvalidAccountIndex,
    DerivationError,
    SyncError,

    // BDK Errors
    Generic,
    NoRecipients,
    NoUtxosSelected,
    OutputBelowDustLimit,
    InsufficientFunds,
    BnBTotalTriesExceeded,
    BnBNoExactMatch,
    UnknownUtxo,
    TransactionNotFound,
    TransactionConfirmed,
    IrreplaceableTransaction,
    FeeRateTooLow,
    FeeTooLow,
    FeeRateUnavailable,
    MissingKeyOrigin,
    Key,
    ChecksumMismatch,
    SpendingPolicyRequired,
    InvalidPolicyPathError,
    Signer,
    InvalidOutpoint,
    Descriptor,
    Miniscript,
    MiniscriptPsbt,
    Bip32,
    Psbt,
}

#[wasm_bindgen(getter_with_clone)]
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

impl Into<DetailledWasmError> for Error {
    fn into(self) -> DetailledWasmError {
        match self {
            Error::InvalidSecretKey => DetailledWasmError {
                kind: WasmError::InvalidSecretKey,
                details: JsValue::null(),
            },
            Error::InvalidDescriptor => DetailledWasmError {
                kind: WasmError::InvalidDescriptor,
                details: JsValue::null(),
            },
            Error::InvalidDerivationPath => DetailledWasmError {
                kind: WasmError::InvalidDerivationPath,
                details: JsValue::null(),
            },
            Error::InvalidAccountIndex => DetailledWasmError {
                kind: WasmError::InvalidAccountIndex,
                details: JsValue::null(),
            },
            Error::DerivationError => DetailledWasmError {
                kind: WasmError::DerivationError,
                details: JsValue::null(),
            },
            Error::SyncError => DetailledWasmError {
                kind: WasmError::SyncError,
                details: JsValue::null(),
            },
            Error::Generic { msg } => DetailledWasmError {
                kind: WasmError::Generic,
                details: JsValue::from_str(&msg),
            },
            Error::NoRecipients => DetailledWasmError {
                kind: WasmError::NoRecipients,
                details: JsValue::null(),
            },
            Error::NoUtxosSelected => DetailledWasmError {
                kind: WasmError::NoUtxosSelected,
                details: JsValue::null(),
            },
            Error::OutputBelowDustLimit { output } => DetailledWasmError {
                kind: WasmError::OutputBelowDustLimit,
                details: output.into(),
            },
            Error::InsufficientFunds { needed, available } => DetailledWasmError {
                kind: WasmError::InsufficientFunds,
                details: serde_wasm_bindgen::to_value(&(needed, available)).unwrap(),
            },
            Error::BnBTotalTriesExceeded => DetailledWasmError {
                kind: WasmError::BnBTotalTriesExceeded,
                details: JsValue::null(),
            },
            Error::BnBNoExactMatch => DetailledWasmError {
                kind: WasmError::BnBNoExactMatch,
                details: JsValue::null(),
            },
            Error::UnknownUtxo => DetailledWasmError {
                kind: WasmError::UnknownUtxo,
                details: JsValue::null(),
            },
            Error::TransactionNotFound => DetailledWasmError {
                kind: WasmError::TransactionNotFound,
                details: JsValue::null(),
            },
            Error::TransactionConfirmed => DetailledWasmError {
                kind: WasmError::TransactionConfirmed,
                details: JsValue::null(),
            },
            Error::IrreplaceableTransaction => DetailledWasmError {
                kind: WasmError::IrreplaceableTransaction,
                details: JsValue::null(),
            },
            Error::FeeRateTooLow { required } => DetailledWasmError {
                kind: WasmError::FeeRateTooLow,
                details: JsValue::from_str(&required),
            },
            Error::FeeTooLow { required } => DetailledWasmError {
                kind: WasmError::FeeTooLow,
                details: JsValue::from_f64(required as f64),
            },
            Error::FeeRateUnavailable => DetailledWasmError {
                kind: WasmError::FeeRateUnavailable,
                details: JsValue::null(),
            },
            Error::MissingKeyOrigin { key } => DetailledWasmError {
                kind: WasmError::MissingKeyOrigin,
                details: JsValue::from_str(&key),
            },
            Error::Key { error } => DetailledWasmError {
                kind: WasmError::Key,
                details: JsValue::from_str(&error),
            },
            Error::ChecksumMismatch => DetailledWasmError {
                kind: WasmError::ChecksumMismatch,
                details: JsValue::null(),
            },
            Error::SpendingPolicyRequired { keychain_kind } => DetailledWasmError {
                kind: WasmError::SpendingPolicyRequired,
                details: JsValue::from_str(match keychain_kind {
                    KeychainKind::External => "External",
                    KeychainKind::Internal => "Internal",
                }),
            },
            Error::InvalidPolicyPathError { error } => DetailledWasmError {
                kind: WasmError::InvalidPolicyPathError,
                details: JsValue::from_str(&error),
            },
            Error::Signer { error } => DetailledWasmError {
                kind: WasmError::Signer,
                details: JsValue::from_str(&error),
            },
            Error::InvalidOutpoint { outpoint } => DetailledWasmError {
                kind: WasmError::InvalidOutpoint,
                details: JsValue::from_str(&outpoint),
            },
            Error::Descriptor { error } => DetailledWasmError {
                kind: WasmError::Descriptor,
                details: JsValue::from_str(&error),
            },
            Error::Miniscript { error } => DetailledWasmError {
                kind: WasmError::Miniscript,
                details: JsValue::from_str(&error),
            },
            Error::MiniscriptPsbt => DetailledWasmError {
                kind: WasmError::MiniscriptPsbt,
                details: JsValue::null(),
            },
            Error::Bip32 { error } => DetailledWasmError {
                kind: WasmError::Bip32,
                details: JsValue::from_str(&error),
            },
            Error::Psbt { error } => DetailledWasmError {
                kind: WasmError::Psbt,
                details: JsValue::from_str(&error),
            },
        }
    }
}