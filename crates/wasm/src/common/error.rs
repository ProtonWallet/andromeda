use std::error::Error;

use andromeda_api::error::Error as ApiError;
use andromeda_bitcoin::error::{
    Bip39Error, BitcoinAddressParseError, CreateTxError, Error as BitcoinError, ExtractTxError, InsufficientFundsError,
};
use andromeda_common::error::Error as CommonError;
use andromeda_esplora::error::Error as EsploraError;
use andromeda_features::error::Error as FeaturesError;
use serde::Serialize;
use serde_json::{json, Value};
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::JsValue;

pub trait ErrorExt {
    fn to_js_error(self) -> JsValue;
}

pub fn json_to_jsvalue(json: Value) -> JsValue {
    // Serialize the Rust struct into a JsValue
    let serializer = Serializer::new().serialize_maps_as_objects(true);
    json.serialize(&serializer).unwrap()
}

impl ErrorExt for ApiError {
    fn to_js_error(self) -> JsValue {
        match self {
            ApiError::AuthSession(kind) => JsValue::from(&format!(
                "AuthSession: A muon {kind} error was caused by a non-existent auth session",
            )),
            ApiError::AuthRefresh(kind) => JsValue::from(&format!(
                "AuthRefresh: A muon {kind} error was caused by a failed auth refresh"
            )),
            ApiError::ForkAuthSession => JsValue::from(
                &"ForkAuthSession: A muon error was caused by a failed auth via forked session".to_string(),
            ),
            ApiError::ForkSession => {
                JsValue::from(&"ForkSession: A muon error was caused by a failed fork session".to_string())
            }
            ApiError::LoginError => JsValue::from(&"LoginError: A muon error was caused by a failed login".to_string()),
            ApiError::UnsupportedTwoFactor => {
                JsValue::from(&"UnsupportedTwoFactor: A muon error was caused by unsupported TwoFactor".to_string())
            }
            ApiError::MuonError(me) => JsValue::from(&format!(
                "MuonError: {me} (caused by: {source:?})",
                source = me.source(),
            )),
            ApiError::BitcoinDeserialize(bde) => {
                JsValue::from(&format!("BitcoinDeserializeError occurred: {:?}", bde.source()))
            }
            ApiError::HexToArrayDecoding(hde) => {
                JsValue::from(&format!("HexToArrayDecoding error occurred: {:?}", hde.source()))
            }
            ApiError::HexToBytesErrorDecoding(hde) => {
                JsValue::from(&format!("HexToBytesErrorDecoding error occurred: {:?}", hde.source()))
            }
            ApiError::Http => JsValue::from("HTTP error occurred"),
            ApiError::ErrorCode(status, error) => json_to_jsvalue(json!({
                "kind": "HTTPError",
                "status": status.as_u16(),
                "code": error.Code,
                "error": error.Error,
                "details": error.Details
            })),
            ApiError::Deserialize(err) => JsValue::from(&err),
            ApiError::MuonAppVersion(err) => JsValue::from(&format!("MuonAppVersion occurred: {:?}", err.source())),
            ApiError::MuonStatus(err) => JsValue::from(&format!("MuonStatusError occurred: {:?}", err.source())),
            ApiError::Utf8Error(err) => JsValue::from(&format!("Utf8Error occurred: {:?}", err.source())),
        }
    }
}

impl ErrorExt for BitcoinError {
    fn to_js_error(self) -> JsValue {
        let common_error = JsValue::from(
            &format!("Wasm error occurred in Bitcoin: {}", self),
            // json!(self), TODO: fix this to have more detailed errors
        );

        match self {
            BitcoinError::InvalidFeeRate => json_to_jsvalue(json!({
                "kind": "InvalidFeeRate",
            })),
            BitcoinError::BitcoinAddressParse(error) => match error {
                BitcoinAddressParseError::Base58(_) => json_to_jsvalue(json!({
                    "kind": "Base58",
                })),
                BitcoinAddressParseError::Bech32(_) => json_to_jsvalue(json!({
                    "kind": "Bech32",
                })),
                BitcoinAddressParseError::NetworkValidation(_) => json_to_jsvalue(json!({
                    "kind": "NetworkValidation",
                })),
                _ => common_error,
            },
            BitcoinError::CreateTx(error) => match error {
                CreateTxError::CoinSelection(InsufficientFundsError { needed, available }) => json_to_jsvalue(json!({
                    "kind": "InsufficientFunds",
                    "needed": needed,
                    "available": available,
                })),
                CreateTxError::OutputBelowDustLimit(limit) => json_to_jsvalue(json!({
                    "kind": "OutputBelowDustLimit",
                    "limit": limit,
                })),
                _ => common_error,
            },
            BitcoinError::ExtractTx(error) => match error {
                ExtractTxError::AbsurdFeeRate { .. } => json_to_jsvalue(json!({
                    "kind": "AbsurdFeeRate",
                })),
                ExtractTxError::MissingInputValue { .. } => json_to_jsvalue(json!({
                    "kind": "MissingInputValue",
                })),
                ExtractTxError::SendingTooMuch { .. } => json_to_jsvalue(json!({
                    "kind": "SendingTooMuch",
                })),
                _ => common_error,
            },
            BitcoinError::Bip39(error) => match error {
                Bip39Error::BadWordCount(count) => json_to_jsvalue(json!({
                    "kind": "BadWordCount",
                    "count": count,
                })),
                Bip39Error::UnknownWord(index) => json_to_jsvalue(json!({
                    "kind": "UnknownWord",
                    "index": index,
                })),
                Bip39Error::BadEntropyBitCount(bit) => json_to_jsvalue(json!({
                    "kind": "BadEntropyBitCount",
                    "bit": bit,
                })),
                Bip39Error::InvalidChecksum => json_to_jsvalue(json!({
                    "kind": "InvalidChecksum",
                })),
                Bip39Error::AmbiguousLanguages(_) => json_to_jsvalue(json!({
                    "kind": "AmbiguousLanguages",
                })),
            },
            BitcoinError::EsploraClient(EsploraError::ApiError(error)) => error.to_js_error(),
            BitcoinError::InsufficientFundsInPaperWallet => json_to_jsvalue(json!({
                "kind": "InsufficientFundsInPaperWallet",
            })),
            BitcoinError::InvalidPaperWallet => json_to_jsvalue(json!({
                "kind": "InvalidPaperWallet",
            })),
            _ => common_error,
        }
    }
}

impl ErrorExt for CommonError {
    fn to_js_error(self) -> JsValue {
        match self {
            CommonError::InvalidNetwork(network) => json_to_jsvalue(json!({
                "kind": "InvalidNetwork",
                "network": network,
            })),
            CommonError::InvalidScriptType(script_type) => json_to_jsvalue(json!({
                "kind":"InvalidScriptType",
                "scriptType": script_type,
            })),
            CommonError::CompileTypst => json_to_jsvalue(json!({
                "kind": "CompileTypst",
            })),
            CommonError::ExportPDF => json_to_jsvalue(json!({
                "kind": "ExportPDF",
            })),
        }
    }
}

impl ErrorExt for FeaturesError {
    fn to_js_error(self) -> JsValue {
        let common_error = JsValue::from(
            &format!("Wasm error occurred in Bitcoin: {}", self),
            // json!(self), TODO: fix this to have more detailed errors
        );

        match self {
            FeaturesError::AndromedaBitcoinError(error) => match error {
                BitcoinError::InvalidFeeRate => json_to_jsvalue(json!({
                    "kind": "InvalidFeeRate",
                })),
                BitcoinError::BitcoinAddressParse(error) => match error {
                    BitcoinAddressParseError::Base58(_) => json_to_jsvalue(json!({
                        "kind": "Base58",
                    })),
                    BitcoinAddressParseError::Bech32(_) => json_to_jsvalue(json!({
                        "kind": "Bech32",
                    })),
                    BitcoinAddressParseError::NetworkValidation(_) => json_to_jsvalue(json!({
                        "kind": "NetworkValidation",
                    })),
                    _ => common_error,
                },
                BitcoinError::CreateTx(error) => match error {
                    CreateTxError::CoinSelection(InsufficientFundsError { needed, available }) => {
                        json_to_jsvalue(json!({
                            "kind": "InsufficientFunds",
                            "needed": needed,
                            "available": available,
                        }))
                    }
                    CreateTxError::OutputBelowDustLimit(limit) => json_to_jsvalue(json!({
                        "kind": "OutputBelowDustLimit",
                        "limit": limit,
                    })),
                    _ => common_error,
                },
                BitcoinError::ExtractTx(error) => match error {
                    ExtractTxError::AbsurdFeeRate { .. } => json_to_jsvalue(json!({
                        "kind": "AbsurdFeeRate",
                    })),
                    ExtractTxError::MissingInputValue { .. } => json_to_jsvalue(json!({
                        "kind": "MissingInputValue",
                    })),
                    ExtractTxError::SendingTooMuch { .. } => json_to_jsvalue(json!({
                        "kind": "SendingTooMuch",
                    })),
                    _ => common_error,
                },
                BitcoinError::Bip39(error) => match error {
                    Bip39Error::BadWordCount(count) => json_to_jsvalue(json!({
                        "kind": "BadWordCount",
                        "count": count,
                    })),
                    Bip39Error::UnknownWord(index) => json_to_jsvalue(json!({
                        "kind": "UnknownWord",
                        "index": index,
                    })),
                    Bip39Error::BadEntropyBitCount(bit) => json_to_jsvalue(json!({
                        "kind": "BadEntropyBitCount",
                        "bit": bit,
                    })),
                    Bip39Error::InvalidChecksum => json_to_jsvalue(json!({
                        "kind": "InvalidChecksum",
                    })),
                    Bip39Error::AmbiguousLanguages(_) => json_to_jsvalue(json!({
                        "kind": "AmbiguousLanguages",
                    })),
                },
                BitcoinError::EsploraClient(EsploraError::ApiError(error)) => error.to_js_error(),
                BitcoinError::InsufficientFundsInPaperWallet => json_to_jsvalue(json!({
                    "kind": "InsufficientFundsInPaperWallet",
                })),
                BitcoinError::InvalidPaperWallet => json_to_jsvalue(json!({
                    "kind": "InvalidPaperWallet",
                })),
                _ => common_error,
            },
            FeaturesError::AndromedaCommonError(error) => match error {
                CommonError::InvalidNetwork(network) => json_to_jsvalue(json!({
                    "kind": "InvalidNetwork",
                    "network": network,
                })),
                CommonError::InvalidScriptType(script_type) => json_to_jsvalue(json!({
                    "kind":"InvalidScriptType",
                    "scriptType": script_type,
                })),
                CommonError::CompileTypst => json_to_jsvalue(json!({
                    "kind": "CompileTypst",
                })),
                CommonError::ExportPDF => json_to_jsvalue(json!({
                    "kind": "ExportPDF",
                })),
            },
            FeaturesError::AccountExportDatetimeError => json_to_jsvalue(json!({
                "kind": "AccountExportDatetimeError",
            })),
        }
    }
}
