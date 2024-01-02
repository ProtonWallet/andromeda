mod types;

uniffi::include_scaffolding!("common");

pub use types::{address_index::AddressIndex, address_info::AddressInfo};

pub use proton_wallet_common::account::{Account, AccountConfig};
pub use proton_wallet_common::bitcoin::Network;
pub use proton_wallet_common::error::Error;
pub use proton_wallet_common::mnemonic::Mnemonic;
pub use proton_wallet_common::transaction_builder::TxBuilder;
pub use proton_wallet_common::wallet::{Wallet, WalletConfig};

pub use proton_wallet_common::{
    Balance, DerivationPath, Height, KeychainKind, Language, LockTime, Time, Transaction, WordCount,
};

//
pub fn library_version() -> String {
    proton_wallet_common::library_version()
}
