use bdk::keys::bip39::WordCount;
use proton_wallet_common::keys;

#[test]
fn test_gen_mnemonic() {
    let mnemonic = keys::gen_mnemonic(WordCount::Words12);
    assert!(!mnemonic.is_empty(), "menemonic is empty!");
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert!(words.len() == 12, "menemonic is not 12 words: '{}'", mnemonic);

    let mnemonic = keys::gen_mnemonic(WordCount::Words15);
    assert!(!mnemonic.is_empty(), "menemonic is empty!");
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert!(words.len() == 15, "menemonic is not 15 words: '{}'", mnemonic);

    let mnemonic = keys::gen_mnemonic(WordCount::Words18);
    assert!(!mnemonic.is_empty(), "menemonic is empty!");
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert!(words.len() == 18, "menemonic is not 18 words: '{}'", mnemonic);

    let mnemonic = keys::gen_mnemonic(WordCount::Words21);
    assert!(!mnemonic.is_empty(), "menemonic is empty!");
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert!(words.len() == 21, "menemonic is not 21 words: '{}'", mnemonic);

    let mnemonic = keys::gen_mnemonic(WordCount::Words24);
    assert!(!mnemonic.is_empty(), "menemonic is empty!");
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    assert!(words.len() == 24, "menemonic is not 24 words: '{}'", mnemonic);
}

#[test]
fn test_gen_address() {
    let phrase: String = keys::gen_mnemonic(WordCount::Words12);
    print!("phrase: {}", phrase);
    let password = "test_pass";
    let address = keys::gen_address(&phrase, password);

    print!("address: {}", address);
    assert!(!address.is_empty(), "Hello, world!");
}
