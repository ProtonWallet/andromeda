use wasm_bindgen::prelude::*;
use proton_wallet_common::Transaction;
use super::locktime::WasmLockTime;


#[wasm_bindgen]
pub struct WasmBdkTransaction {
    version: i32,
    lock_time: WasmLockTime, 
    // input: Vec<WasmTxIn>,  //TODO
    // output: Vec<WasmTxOut>, //TODO
}

impl Into<WasmBdkTransaction> for Transaction {
    fn into(self) -> WasmBdkTransaction {
        WasmBdkTransaction {
            version: self.version(),
            lock_time: self.get_inner().lock_time.into(),
            // TODO add input output
        }
    }
}

#[wasm_bindgen]
pub struct WasmTransaction {
    inner: Transaction,
}

#[wasm_bindgen]
impl WasmTransaction {
    #[wasm_bindgen(constructor)]
    pub fn new(transaction_bytes: Vec<u8>) -> Result<WasmTransaction, JsValue> {
        let tx = Transaction::new(transaction_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(WasmTransaction { inner: tx })
    }

    pub fn txid(&self) -> String {
        self.inner.txid().to_string()
    }

    pub fn size(&self) -> u64 {
        self.inner.size() as u64
    }

    pub fn vsize(&self) -> u64 {
        self.inner.vsize() as u64
    }

    pub fn is_coin_base(&self) -> bool {
        self.inner.is_coin_base()
    }

    pub fn is_explicitly_rbf(&self) -> bool {
        self.inner.is_explicitly_rbf()
    }

    pub fn is_lock_time_enabled(&self) -> bool {
        self.inner.is_lock_time_enabled()
    }

    pub fn version(&self) -> i32 {
        self.inner.version() as i32
    }
}
