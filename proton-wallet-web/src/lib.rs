mod account;
mod address;
mod address_index;
mod address_info;
mod balance;
mod common;
mod defined;
mod derivation_path;
mod descriptor;
pub mod error;
mod keys;
mod locktime;
mod mnemonic;
mod partially_signed_transaction;
mod script;
mod transaction;
mod tx_builder;
mod utils;
mod wallet;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn library_version() -> String {
    proton_wallet_common::library_version()
}

pub use utils::set_panic_hook;
