use proton_wallet_common::{AddressInfo as CommonAddressInfo, KeychainKind};

/// We reimplement AddressIndex because udl only support struct variant enums
#[derive(Debug)]
pub struct AddressInfo {
    pub index: u32,
    pub keychain: KeychainKind,
}

impl Into<AddressInfo> for CommonAddressInfo {
    fn into(self) -> AddressInfo {
        AddressInfo {
            index: self.index,
            keychain: self.keychain,
        }
    }
}
