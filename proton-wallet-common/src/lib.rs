pub mod common;

pub mod onchain;

pub use bdk::{
    bitcoin::{
        bip32::{DerivationPath, ExtendedPrivKey},
        blockdata::locktime::absolute::{Height, LockTime, Time},
        psbt::PartiallySignedTransaction,
        Address, Network as BdkNetwork, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
    },
    chain::{ChainPosition, ConfirmationTime, ConfirmationTimeAnchor},
    keys::{
        bip39::{Language, Mnemonic, MnemonicWithPassphrase, WordCount},
        DerivableKey, ExtendedKey,
    },
    wallet::{tx_builder::ChangeSpendPolicy, AddressIndex, AddressInfo, Balance, ChangeSet},
    KeychainKind, LocalUtxo, SignOptions,
};

pub use bdk_chain::{Append, PersistBackend};

pub fn library_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
