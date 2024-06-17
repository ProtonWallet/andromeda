use bdk_persist::PersistBackend;
use bdk_wallet::wallet::ChangeSet;

pub trait PersistBackendFactory<P>
where
    P: PersistBackend<ChangeSet>,
{
    fn build(self, key: String) -> P;
}

impl PersistBackendFactory<()> for () {
    fn build(self, _key: String) {}
}
