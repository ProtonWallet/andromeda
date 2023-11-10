mod address;
mod defined;
mod derivation_path;
mod keys;
mod mnemonic;
mod public_key;
mod secret_key;
mod utils;
mod descriptor;
mod transaction;
mod locktime;
mod partially_signed_transaction;
mod common;
mod script;
mod address_index;
mod address_info;
mod tx_builder;
mod wallet;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn library_version() -> String {
    proton_wallet_common::library_version()
}

pub use utils::set_panic_hook;
