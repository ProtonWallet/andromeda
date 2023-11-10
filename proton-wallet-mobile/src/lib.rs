#![allow(clippy::new_without_default)]

mod defines;
mod keys;

//
use proton_wallet_common::Address;
use proton_wallet_common::KeychainKind;
use proton_wallet_common::Script;
use proton_wallet_common::Transaction;
use proton_wallet_common::Update;
use proton_wallet_common::Wallet;
use proton_wallet_common::WordCount;
// use proton_wallet_common::EsploraClient;
use proton_wallet_common::mnemonic::Mnemonic;
use proton_wallet_common::AddressIndex;
use proton_wallet_common::AddressInfo;
use proton_wallet_common::Balance;
use proton_wallet_common::BdkError;
use proton_wallet_common::Descriptor;
use proton_wallet_common::DescriptorPublicKey;
use proton_wallet_common::DescriptorSecretKey;
use proton_wallet_common::TxBuilder;
// use proton_wallet_common::ScriptAmount;
use proton_wallet_common::PartiallySignedTransaction;

uniffi::include_scaffolding!("common");

//
pub fn library_version() -> String {
    proton_wallet_common::library_version()
}
