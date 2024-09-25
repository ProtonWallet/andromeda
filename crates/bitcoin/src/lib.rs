pub mod account;
pub mod bdk_wallet_ext;
pub mod blockchain_client;
pub mod error;
pub mod mnemonic;
pub mod payment_link;
pub mod psbt;
pub mod transaction_builder;
pub mod transactions;
pub mod utils;
pub mod wallet;

pub mod storage;

#[cfg(feature = "sqlite")]
pub use bdk_wallet::rusqlite::Connection;

#[doc(hidden)]
pub use bdk_wallet::{
    bitcoin::{
        bip32::{ChildNumber, DerivationPath, Xpriv},
        block::Header as BlockHeader,
        blockdata::{
            constants::genesis_block,
            locktime::absolute::{Height, LockTime, Time},
        },
        consensus::Params as ConsensusParams,
        Address, Amount, BlockHash, Network as BdkNetwork, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
        Witness,
    },
    chain::{ConfirmationBlockTime, ConfirmationTime},
    keys::{
        bip39::{Language as BdkLanguage, Mnemonic as BdkMnemonic, MnemonicWithPassphrase, WordCount},
        DerivableKey, ExtendedKey,
    },
    tx_builder::ChangeSpendPolicy,
    AddressInfo, Balance, ChangeSet, KeychainKind, LocalOutput, SignOptions,
};
