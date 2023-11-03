mod utils;
mod address;
mod keys;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn library_version() -> String {
    proton_wallet_common::library_version()
}

pub use utils::set_panic_hook;
