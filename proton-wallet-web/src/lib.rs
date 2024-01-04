mod common;
mod onchain;
mod utils;

use wasm_bindgen::prelude::*;

pub use utils::panic_hook::set_panic_hook;

#[wasm_bindgen]
pub fn library_version() -> String {
    proton_wallet_common::library_version()
}
