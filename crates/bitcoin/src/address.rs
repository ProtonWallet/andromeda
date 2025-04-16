use bdk_wallet::{Balance, KeychainKind};

use crate::transactions::TransactionDetails;

#[derive(Clone)]
pub struct AddressDetails {
    pub index: u32,
    pub address: String,
    pub transactions: Vec<TransactionDetails>,
    pub balance: Balance,
    pub keychain: KeychainKind,
}
