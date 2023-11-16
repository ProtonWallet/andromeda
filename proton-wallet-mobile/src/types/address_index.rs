use proton_wallet_common::AddressIndex as CommonAddressIndex;

/// We reimplement AddressIndex because udl only support struct variant enums
#[derive(Debug)]
pub enum AddressIndex {
    New,
    LastUnused,
    Peek { index: u32 },
}

impl Into<AddressIndex> for CommonAddressIndex {
    fn into(self) -> AddressIndex {
        match self {
            CommonAddressIndex::New => AddressIndex::New,
            CommonAddressIndex::LastUnused => AddressIndex::LastUnused,
            CommonAddressIndex::Peek(index) => AddressIndex::Peek { index },
        }
    }
}
