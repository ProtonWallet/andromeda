use andromeda_bitcoin::{Height, LockTime, Time};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmLockTime {
    lock_time: LockTime,
}

impl Into<WasmLockTime> for LockTime {
    fn into(self) -> WasmLockTime {
        WasmLockTime { lock_time: self }
    }
}

impl From<WasmLockTime> for LockTime {
    fn from(value: WasmLockTime) -> Self {
        value.lock_time
    }
}

#[wasm_bindgen]
impl WasmLockTime {
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

    #[wasm_bindgen(js_name = isBlockHeight)]
    pub fn is_block_height(&self) -> bool {
        matches!(self.lock_time, LockTime::Blocks(_))
    }

    #[wasm_bindgen(js_name = isBlockTime)]
    pub fn is_block_time(&self) -> bool {
        matches!(self.lock_time, LockTime::Seconds(_))
    }

    #[wasm_bindgen(js_name = toConsensusU32)]
    pub fn to_consensus_u32(&self) -> u32 {
        match self.lock_time {
            LockTime::Blocks(height) => height.to_consensus_u32(),
            LockTime::Seconds(seconds) => seconds.to_consensus_u32(),
        }
    }
}
