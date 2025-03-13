use andromeda_common::{async_trait_impl, utils::now};
use async_std::sync::{RwLockReadGuard, RwLockWriteGuard};
use bdk_wallet::{PersistedWallet, Update};
use bitcoin::{constants::genesis_block, Address};

use crate::{error::Error, storage::WalletStorage};

/// - **On native platforms**: Requires `Sync + Send` for `AccessWallet`.
/// - **On WebAssembly (wasm32)**: Excludes `Sync + Send` since threading is unsupported.
#[cfg(target_arch = "wasm32")]
pub type AccessWalletDyn = dyn AccessWallet;
#[cfg(not(target_arch = "wasm32"))]
pub type AccessWalletDyn = dyn AccessWallet + Sync + Send;

// Define the AccessWallet trait
async_trait_impl! {
pub trait AccessWallet {
    /// Returns a readable lock to the account's BdkWallet struct.
    async fn lock_wallet(&self) -> RwLockReadGuard<PersistedWallet<WalletStorage>>;

    /// Returns a mutable lock to the account's BdkWallet struct.
    async fn lock_wallet_mut(&self) -> RwLockWriteGuard<PersistedWallet<WalletStorage>>;

    /// Returns a mutable lock to the account's BdkWallet struct.
    async fn lock_persister_mut(&self) -> RwLockWriteGuard<WalletStorage>;

    /// Applies an update to the wallet and persists it.
    async fn apply_update(&self, update: Update) -> Result<(), Error> {
        {
            let mut wallet_lock = self.lock_wallet_mut().await;
            wallet_lock.apply_update_at(update, now().as_secs())?;
        } // lock auto released

        self.persist().await?;
        Ok(())
    }

    /// Returns true if the wallet has been synced at least once.
    async fn has_sync_data(&self) -> bool {
        let wallet_lock = self.lock_wallet().await;
        wallet_lock.latest_checkpoint().hash() != genesis_block(wallet_lock.network()).block_hash()
    }

    /// Returns a boolean indicating whether or not the account owns the provided address
    async fn owns(&self, address: &Address) -> bool {
        self.lock_wallet().await.is_mine(address.script_pubkey())
    }

    /// Returns a boolean indicating whether or not the account has any valid transactions
    async fn has_transactions(&self) -> bool {
        self.lock_wallet().await.transactions().count() > 0
    }

    /// Persist the current wallet state.
    async fn persist(&self) -> Result<(), Error> {
        // Acquire wallet lock inside the function
        let mut wallet_lock = self.lock_wallet_mut().await;

        // Acquire persister lock only when needed
        let mut persister = self.lock_persister_mut().await;

        wallet_lock.persist(&mut *persister).map_err(|_e| Error::PersistError)?;

        Ok(())
    }
}}
