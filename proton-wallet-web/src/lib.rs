mod address;
mod defined;
mod derivation_path;
mod keys;
mod mnemonic;
mod public_key;
mod secret_key;
mod utils;
mod descriptor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn library_version() -> String {
    proton_wallet_common::library_version()
}

pub use utils::set_panic_hook;
