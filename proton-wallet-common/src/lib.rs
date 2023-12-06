mod keys;

// pub mod
pub mod account;
pub mod bitcoin;
pub mod client;
pub mod error;
pub mod mnemonic;
pub mod transaction_builder;
pub mod wallet;

pub use bdk::{
    bitcoin::{
        bip32::{DerivationPath, ExtendedPrivKey},
        blockdata::locktime::absolute::{Height, LockTime, Time},
        psbt::PartiallySignedTransaction,
        OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
    },
    chain::ChainPosition,
    keys::{
        bip39::{Language, Mnemonic, MnemonicWithPassphrase, WordCount},
        DerivableKey, ExtendedKey,
    },
    wallet::{AddressIndex, AddressInfo, Balance},
    KeychainKind,
};

pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
