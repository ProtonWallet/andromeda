use wasm_bindgen::prelude::*;
use proton_wallet_common::{LockTime, Height, Time};
// #[wasm_bindgen]  //could conver to struct with u32 fields
// pub enum WasmLocktTime {
//     Blocks(u32),
//     Seconds(u32),
// }

#[wasm_bindgen]
pub struct WasmLockTime {
    lock_time: LockTime,
}

impl Into<WasmLockTime> for LockTime {
    fn into(self) -> WasmLockTime {
        WasmLockTime { lock_time: self }
    }
}

#[wasm_bindgen]
impl WasmLockTime {
    // #[wasm_bindgen(constructor)]
    // pub fn new() -> WasmLockTime {
    //     unimplemented!()
    // }
    
    #[wasm_bindgen(js_name = fromHeight)]
    pub fn from_height(height: u32) -> WasmLockTime {
        WasmLockTime {
            lock_time: LockTime::Blocks(Height::from_consensus(height).unwrap()),
        }
    }

    #[wasm_bindgen(js_name = fromSeconds)]
    pub fn from_seconds(seconds: u32) -> WasmLockTime {
        WasmLockTime {
            lock_time: LockTime::Seconds(Time::from_consensus(seconds).unwrap()),
        }
    }

    pub fn is_block_height(&self) -> bool {
        matches!(self.lock_time, LockTime::Blocks(_))
    }

    pub fn is_block_time(&self) -> bool {
        matches!(self.lock_time, LockTime::Seconds(_))
    }

    pub fn to_consensus_u32(&self) -> u32 {
        match self.lock_time {
            LockTime::Blocks(height) => height.to_consensus_u32(),
            LockTime::Seconds(seconds) => seconds.to_consensus_u32(),
        }
    }
}