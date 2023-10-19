
use proton_wallet_common::hello::helloworld;

#[test]
fn test_hello_world() {
    let str = helloworld();
    assert!(!str.is_empty(), "Hello, world!");
}
