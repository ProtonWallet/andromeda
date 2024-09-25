use std::{convert::Infallible, fmt::Debug};

pub use bdk_wallet::{chain::Merge, ChangeSet, WalletPersister};

pub trait WalletConnectorFactory<C, P>: Clone + Debug
where
    C: WalletPersisterConnector<P>,
    P: WalletPersister,
{
    fn build(self, key: String) -> C;
}

impl WalletConnectorFactory<MemoryPersisted, MemoryPersisted> for MemoryPersisted {
    fn build(self, _key: String) -> MemoryPersisted {
        MemoryPersisted {}
    }
}

pub trait WalletPersisterConnector<P>: Clone + Debug
where
    P: WalletPersister,
{
    fn connect(&self) -> P;
}

impl WalletPersisterConnector<MemoryPersisted> for MemoryPersisted {
    fn connect(&self) -> MemoryPersisted {
        MemoryPersisted {}
    }
}

#[derive(Clone, Debug)]
pub struct MemoryPersisted;

impl WalletPersister for MemoryPersisted {
    type Error = Infallible;

    fn initialize(_persister: &mut Self) -> Result<ChangeSet, Self::Error> {
        Ok(ChangeSet::default())
    }

    fn persist(_persister: &mut Self, _changeset: &ChangeSet) -> Result<(), Self::Error> {
        Ok(())
    }
}
