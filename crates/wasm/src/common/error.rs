use andromeda_api::error::Error as ApiError;
use andromeda_bitcoin::error::Error as BitcoinError;
use andromeda_common::error::Error as CommonError;

pub trait ErrorExt {
    fn to_js_error(self) -> js_sys::Error;
}

impl ErrorExt for BitcoinError {
    fn to_js_error(self) -> js_sys::Error {
        js_sys::Error::new(
            &format!("Wasm error occured in Bitcoin: {}", self),
            // json!(self), TODO: fix this to have more detailled errors
        )
    }
}

impl ErrorExt for ApiError {
    fn to_js_error(self) -> js_sys::Error {
        js_sys::Error::new(
            &format!("Wasm error occured in API: {}", self),
            // json!(self), TODO: fix this to have more detailled errors
        )
    }
}

impl ErrorExt for CommonError {
    fn to_js_error(self) -> js_sys::Error {
        js_sys::Error::new(
            &format!("Wasm error occured in common code: {}", self),
            // json!(self), TODO: fix this to have more detailled errors
        )
    }
}
