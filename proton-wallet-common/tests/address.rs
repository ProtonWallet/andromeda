
use proton_wallet_common::address::test_address;

#[test]
fn test_test_address() {
    let str = test_address();
    assert!(!str.is_empty(), "Hello, world!");
}
