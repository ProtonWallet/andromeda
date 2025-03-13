use std::sync::Arc;

use andromeda_bitcoin::paper_account::PaperAccount;
use andromeda_common::Network;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::common::{
    error::ErrorExt,
    types::{WasmNetwork, WasmScriptType},
};

/// A representation of a paper wallet account
#[wasm_bindgen]
pub struct WasmPaperAccount {
    inner: Arc<PaperAccount>,
}

impl WasmPaperAccount {
    pub fn get_inner(&self) -> Arc<PaperAccount> {
        self.inner.clone()
    }
}

impl From<Arc<PaperAccount>> for WasmPaperAccount {
    fn from(val: Arc<PaperAccount>) -> Self {
        WasmPaperAccount { inner: val.clone() }
    }
}

#[wasm_bindgen]
impl WasmPaperAccount {
    #[wasm_bindgen(js_name = generate)]
    pub fn generate(network: WasmNetwork, script_type: WasmScriptType) -> Result<Self, js_sys::Error> {
        let paper = Arc::new(PaperAccount::generate(network.into(), script_type.into()).map_err(|e| e.to_js_error())?);
        Ok(paper.into())
    }

    #[wasm_bindgen(js_name = newFrom)]
    pub fn new_from(
        wif: &str,
        script_type: WasmScriptType,
        network: Option<WasmNetwork>,
    ) -> Result<Self, js_sys::Error> {
        let network: Option<Network> = network.map(Into::into);
        let paper =
            Arc::new(PaperAccount::new_from(wif, script_type.into(), network, None).map_err(|e| e.to_js_error())?);
        Ok(paper.into())
    }

    #[wasm_bindgen(js_name = geWif)]
    pub async fn get_wif(&self) -> Result<String, js_sys::Error> {
        Ok(self.get_inner().get_wif().await.map_err(|e| e.to_js_error())?)
    }

    #[wasm_bindgen(js_name = getWifAddress)]
    pub async fn get_wif_address(&self) -> String {
        self.get_inner().get_wif_address().await
    }
}
