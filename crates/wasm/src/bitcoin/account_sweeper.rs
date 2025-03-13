use andromeda_bitcoin::account_sweeper::AccountSweeper;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::common::{error::ErrorExt, types::WasmNetwork};

use super::{account::WasmAccount, blockchain_client::WasmBlockchainClient, psbt::WasmPsbt};

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmAccountSweeper {
    inner: AccountSweeper,
}

#[wasm_bindgen]
impl WasmAccountSweeper {
    #[wasm_bindgen(constructor)]
    pub fn new(client: &WasmBlockchainClient, account: &WasmAccount) -> Self {
        Self {
            inner: AccountSweeper::new(client.into(), account.get_inner()),
        }
    }
}

#[wasm_bindgen]
impl WasmAccountSweeper {
    #[wasm_bindgen(js_name = getSweepWifPsbt)]
    pub async fn get_sweep_wif_psbt(
        &self,
        wif: &str,
        sat_per_vb: u64,
        receive_address_index: u32,
        network: WasmNetwork,
    ) -> Result<WasmPsbt, js_sys::Error> {
        let (psbt, address) = self
            .inner
            .get_sweep_wif_psbt(wif, sat_per_vb, Some(receive_address_index))
            .await
            .map_err(|e| e.to_js_error())?;
        Ok(WasmPsbt::from_paper_account_psbt(&psbt, network.into(), address)?)
    }
}
