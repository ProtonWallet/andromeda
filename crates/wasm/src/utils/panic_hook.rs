use andromeda_api::{network::NetworkClient, utils::common_session};
use wasm_bindgen::prelude::*;

use crate::{bitcoin::types::defined::WasmNetwork, common::error::WasmError};

#[wasm_bindgen(js_name = "setPanicHook")]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = "getNetwork")]
pub async fn get_network() -> Result<WasmNetwork, WasmError> {
    let session = common_session().await;
    let client = NetworkClient::new(session);

    let network = client.get_network().await.unwrap();
    network.try_into()
}
