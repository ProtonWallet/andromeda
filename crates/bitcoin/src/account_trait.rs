use andromeda_common::async_trait_impl;
use async_std::sync::{RwLockReadGuard, RwLockWriteGuard};
use bdk_wallet::PersistedWallet;

use crate::storage::WalletStorage;

// Define the AccessWallet trait
async_trait_impl! {
pub trait AccessWallet {
    /// Returns a readable lock to the account's BdkWallet struct.
    async fn get_wallet(&self) -> RwLockReadGuard<PersistedWallet<WalletStorage>>;

    /// Returns a mutable lock to the account's BdkWallet struct.
    async fn get_mutable_wallet(&self) -> RwLockWriteGuard<PersistedWallet<WalletStorage>>;
}}
