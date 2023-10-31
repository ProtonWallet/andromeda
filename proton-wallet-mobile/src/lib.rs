#![allow(clippy::new_without_default)]

mod hello;
mod keys;
mod address;

uniffi::include_scaffolding!("common");

pub fn library_version() -> String {
    proton_wallet_common::library_version()
}

pub use hello::*;
pub use keys::*;
pub use address::*;