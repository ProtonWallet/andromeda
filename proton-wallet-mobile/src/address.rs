pub use proton_wallet_common::address::test_address;
pub struct Address;
impl Address {
    pub fn new() -> Self {
        Self
    }

    pub fn test_address(&self) -> String {
        return test_address();
    }
}
