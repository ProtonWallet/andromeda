use std::error::Error;

use andromeda_api::error::Error as ApiError;
use andromeda_bitcoin::error::{CoinSelectionError, CreateTxError, Error as BitcoinError};
use andromeda_common::error::Error as CommonError;
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

impl ErrorExt for BitcoinError {
    fn to_js_error(self) -> JsValue {
        let common_error = JsValue::from(
            &format!("Wasm error occured in Bitcoin: {}", self),
            // json!(self), TODO: fix this to have more detailled errors
        );

        match self {
            BitcoinError::CreateTx(error) => match error {
                CreateTxError::CoinSelection(CoinSelectionError::InsufficientFunds { needed, available }) => {
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
            _ => common_error,
        }
    }
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
        }
    }
}
