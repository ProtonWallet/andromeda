use wasm_bindgen::prelude::*;

/**
 * wasm-bindgen has some limitations that implies returning jsValue instead of rust type for async methods. We need to use these additional types to avoid completely breaking type system
 *
 * Ideally we want to get rid of the code below when wasm-bindgen completely supports async fns
 */

// Transaction

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmDerivationPath: &'static str = r#"
interface IWasmTransactionTime {
    confirmed: boolean;
    confirmation_time?: BigInt;
    last_seen?: BigInt;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmDerivationPath: &'static str = r#"
interface IWasmDerivationPath {
    inner: {
        to_string: () => string
    }
}
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmSimpleTransaction: &'static str = r#"
interface IWasmSimpleTransaction {
    txid: string;
    value: BigInt;
    fees?: BigInt;
    time: IWasmTransactionTime,
    account_key?: IWasmDerivationPath,
}
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmSimpleTransactionArray: &'static str = r#"
type IWasmSimpleTransactionArray = IWasmSimpleTransaction[]
"#;

// UTXOs

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmAddress: &'static str = r#"
interface IWasmAddress {
    to_string: () => string
}
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmScript: &'static str = r#"
interface IWasmScript {
    to_address: () => IWasmAddress
}
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmKeychainKind: &'static str = r#"
enum IWasmKeychainKind {
    External, 
    Internal
}
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmOutpoint: &'static str = r#"
type IWasmOutpoint = string
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmUtxo: &'static str = r#"
interface IWasmUtxo {
    value: BigInt;
    outpoint: IWasmOutpoint;
    script_pubkey: IWasmScript;
    keychain: IWasmKeychainKind;
    is_spent: boolean;
    derivation_index: BigInt;
    confirmation_time: IWasmTransactionTime;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
pub const IWasmUtxoArray: &'static str = r#"
type IWasmUtxoArray = IWasmUtxo[]
"#;

// Expose

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IWasmOutpoint")]
    pub type IWasmOutpoint;

    #[wasm_bindgen(typescript_type = "IWasmDerivationPath")]
    pub type IWasmDerivationPath;

    #[wasm_bindgen(typescript_type = "IWasmSimpleTransaction")]
    pub type IWasmSimpleTransaction;

    #[wasm_bindgen(typescript_type = "IWasmSimpleTransactionArray")]
    pub type IWasmSimpleTransactionArray;

    #[wasm_bindgen(typescript_type = "IWasmUtxo")]
    pub type IWasmUtxo;

    #[wasm_bindgen(typescript_type = "IWasmUtxoArray")]
    pub type IWasmUtxoArray;
}
