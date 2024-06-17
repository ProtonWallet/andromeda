use bdk_wallet::{
    chain::{CombinedChangeSet, ConfirmationTimeHeightAnchor},
    KeychainKind,
};

use crate::error::Error;

pub type ChangeSet = CombinedChangeSet<KeychainKind, ConfirmationTimeHeightAnchor>;

pub trait WalletStore: Clone {
    fn read(&self) -> Result<Option<ChangeSet>, Error>;

    fn write(&self, changeset: &ChangeSet) -> Result<(), Error>;
}

pub trait WalletStoreFactory<P>
where
    P: WalletStore,
{
    fn build(self, key: String) -> P;
}

impl WalletStoreFactory<()> for () {
    fn build(self, _key: String) {}
}

impl WalletStore for () {
    fn read(&self) -> Result<Option<ChangeSet>, Error> {
        Ok(None)
    }

    fn write(&self, _changeset: &ChangeSet) -> Result<(), Error> {
        Ok(())
    }
}
