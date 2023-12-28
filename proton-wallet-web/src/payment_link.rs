use proton_wallet_common::payment_link::PaymentLink;
use wasm_bindgen::prelude::*;

use crate::{error::DetailledWasmError, types::defined::WasmNetwork};

#[wasm_bindgen]
pub enum WasmPaymentLinkKind {
    BitcoinAddress,
    BitcoinURI,
    LightningURI,
    UnifiedURI,
}

#[wasm_bindgen]
pub struct WasmPaymentLink {
    inner: PaymentLink,
}

impl Into<WasmPaymentLink> for PaymentLink {
    fn into(self) -> WasmPaymentLink {
        WasmPaymentLink { inner: self }
    }
}

impl Into<PaymentLink> for WasmPaymentLink {
    fn into(self) -> PaymentLink {
        self.inner
    }
}

#[wasm_bindgen]
impl WasmPaymentLink {
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.inner.to_string()
    }

    #[wasm_bindgen(js_name = toUri)]
    pub fn to_uri(&self) -> String {
        self.inner.to_uri()
    }

    #[wasm_bindgen(js_name = tryParse)]
    pub fn try_parse(str: String, network: WasmNetwork) -> Result<WasmPaymentLink, DetailledWasmError> {
        let inner = PaymentLink::try_parse(str, network.into()).map_err(|e| e.into())?;

        Ok(WasmPaymentLink { inner })
    }

    #[wasm_bindgen(js_name = getKind)]
    pub fn get_kind(&self) -> WasmPaymentLinkKind {
        match self.inner {
            PaymentLink::BitcoinAddress(_) => WasmPaymentLinkKind::BitcoinAddress,
            PaymentLink::BitcoinURI { .. } => WasmPaymentLinkKind::BitcoinURI,
            PaymentLink::LightningURI { .. } => WasmPaymentLinkKind::LightningURI,
            PaymentLink::UnifiedURI { .. } => WasmPaymentLinkKind::UnifiedURI,
        }
    }
}
