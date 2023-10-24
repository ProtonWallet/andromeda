
use proton_wallet_common::keys;

#[test]
fn test_gen_mnemonic() {
    let str = keys::gen_mnemonic();
    print!("{}", str);
    assert!(!str.is_empty(), "Hello, world!");
}

#[test]
fn test_gen_address() {

    let phrase: String = keys::gen_mnemonic();
    print!("phrase: {}", phrase);
    let password = "test_pass";
    let address = keys::gen_address(&phrase, password);
    
    print!("address: {}", address);
    assert!(!address.is_empty(), "Hello, world!");
}
