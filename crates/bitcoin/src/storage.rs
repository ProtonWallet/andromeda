use std::{fmt::Debug, sync::Arc};

use crate::error::Error;
pub use bdk_wallet::{chain::Merge, ChangeSet, WalletPersister};

use bdk_wallet::serde_json;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::PathBuf,
};

/// storage factory
pub trait WalletPersisterFactory: Clone + Debug {
    fn build(self, key: String) -> Arc<dyn Storage>;
}

/// this is memory storage, mostly for testing purposes, could preload cache to preload wallet
#[derive(Clone, Debug)]
pub struct MemoryPersisted;

impl MemoryPersisted {
    pub fn new_storage() -> Arc<dyn Storage> {
        Arc::new(MemoryPersisted {})
    }
}

impl Storage for MemoryPersisted {
    fn initialize(&self) -> Result<ChangeSet, Error> {
        Ok(ChangeSet::default())
    }
    fn persist(&self, _changeset: &ChangeSet) -> Result<(), Error> {
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct WalletMemoryPersisterFactory;
impl WalletPersisterFactory for WalletMemoryPersisterFactory {
    fn build(self, _key: String) -> Arc<dyn Storage> {
        MemoryPersisted::new_storage()
    }
}

/// Proton wallet Storage base trail
pub trait Storage: Debug + Send + Sync + 'static {
    /* Other methods not included */
    fn initialize(&self) -> Result<ChangeSet, Error>;
    fn persist(&self, changeset: &ChangeSet) -> Result<(), Error>;
}

/// WalletStorage is a wrapper around the storage trait to be used with bdk_wallet
#[derive(Debug, Clone)]
pub struct WalletStorage(pub Arc<dyn Storage>);

impl WalletStorage {
    /// Create a new WalletStorage with a memory only storage
    pub fn memory_persist() -> Self {
        Self(MemoryPersisted::new_storage())
    }
}
impl WalletPersister for WalletStorage {
    type Error = Error;

    fn persist(persister: &mut Self, changeset: &bdk_wallet::ChangeSet) -> Result<(), Self::Error> {
        persister.0.as_ref().persist(changeset)
    }

    fn initialize(persister: &mut Self) -> Result<bdk_wallet::ChangeSet, Self::Error> {
        persister.0.as_ref().initialize()
    }
}

#[derive(Clone, Debug)]
pub struct WalletFilePersisterFactory(pub bool);

impl WalletPersisterFactory for WalletFilePersisterFactory {
    fn build(self, key: String) -> Arc<dyn Storage> {
        Arc::new(WalletFilePersister::new(key, self.0))
    }
}

#[derive(Clone, Debug)]
struct WalletFilePersister {
    changeset_file: PathBuf,
    is_read_only: bool,
}

impl WalletFilePersister {
    fn new(key: String, is_read_only: bool) -> Self {
        const CHANGESET_KEY_BASE: &str = "mock";

        let mut path = PathBuf::from("./src/tests/mocks/wallets/");
        path.push(format!("{}_{}.json", CHANGESET_KEY_BASE, key));

        Self {
            changeset_file: path,
            is_read_only,
        }
    }

    fn get(&self) -> Option<ChangeSet> {
        if !self.changeset_file.exists() {
            return None;
        }
        let mut file = File::open(&self.changeset_file).ok()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok()?;

        println!("contents: {:?}", self.changeset_file);

        serde_json::from_str(&contents).ok()
    }

    fn set(&self, changeset: ChangeSet) -> Result<(), Error> {
        if self.is_read_only {
            return Ok(());
        }
        let serialized = serde_json::to_string_pretty(&changeset).unwrap();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.changeset_file)
            .unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
        Ok(())
    }
}

impl Storage for WalletFilePersister {
    fn initialize(&self) -> Result<ChangeSet, Error> {
        Ok(self.get().unwrap_or_default())
    }
    fn persist(&self, new_changeset: &ChangeSet) -> Result<(), Error> {
        let mut prev_changeset = self.get().unwrap_or_default();
        prev_changeset.merge(new_changeset.clone());
        self.set(prev_changeset)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use andromeda_api::tests::utils::common_api_client;
    use andromeda_common::{Network, ScriptType};

    use crate::{
        account_syncer::AccountSyncer, blockchain_client::BlockchainClient,
        tests::utils::tests::set_test_wallet_account,
    };

    #[tokio::test]
    #[ignore]
    async fn dump_wallet_to_file_1() {
        let account = Arc::new(set_test_wallet_account(
            "your mnemonic here",
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(true),
            Some(false),
            Some(Network::Regtest),
            None,
        ));
        account.mark_receive_addresses_used_to(0, Some(210)).await.unwrap();
        let api_client = common_api_client().await;
        let client = Arc::new(BlockchainClient::new(api_client.clone()));
        let sync = AccountSyncer::new(client, account.clone());
        // do full sync
        sync.full_sync(Some(200)).await.unwrap();

        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(highest.is_some());
    }

    #[tokio::test]
    async fn test_file_storage() {
        let mnemonic = "remove over athlete patient priority unable memory axis sunset home balance sausage";
        let account = set_test_wallet_account(
            mnemonic,
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(false),
            Some(false),
            Some(Network::Regtest),
            None,
        );
        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(highest.is_none());
        let account = set_test_wallet_account(
            mnemonic,
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(true),
            Some(false),
            Some(Network::Regtest),
            None,
        );
        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(highest.unwrap() == 144);

        let account = set_test_wallet_account(
            "remove over athlete patient priority unable memory axis sunset home balance sausage",
            ScriptType::NativeSegwit,
            "m/84'/1'/0'",
            Some(true),
            Some(true),
            Some(Network::Regtest),
            None,
        );
        let highest = account
            .get_highest_used_address_index_in_output(bdk_wallet::KeychainKind::External)
            .await
            .unwrap();
        assert!(highest.is_some());
    }
}
