use andromeda_api::invite::{InviteClient, InviteNotificationType, RemainingMonthlyInvitations};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::common::error::ErrorExt;

#[derive(Tsify, Serialize, Deserialize, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmInviteNotificationType {
    Newcomer,
    EmailIntegration,
    Unsupported,
}

impl From<InviteNotificationType> for WasmInviteNotificationType {
    fn from(value: InviteNotificationType) -> Self {
        match value {
            InviteNotificationType::Newcomer => WasmInviteNotificationType::Newcomer,
            InviteNotificationType::EmailIntegration => WasmInviteNotificationType::EmailIntegration,
            InviteNotificationType::Unsupported => WasmInviteNotificationType::Unsupported,
        }
    }
}

impl From<WasmInviteNotificationType> for InviteNotificationType {
    fn from(value: WasmInviteNotificationType) -> Self {
        match value {
            WasmInviteNotificationType::Newcomer => InviteNotificationType::Newcomer,
            WasmInviteNotificationType::EmailIntegration => InviteNotificationType::EmailIntegration,
            WasmInviteNotificationType::Unsupported => InviteNotificationType::Unsupported,
        }
    }
}
#[wasm_bindgen]
#[allow(non_snake_case)]
pub struct WasmRemainingMonthlyInvitations {
    pub Available: u8,
    pub Used: u8,
}

impl From<RemainingMonthlyInvitations> for WasmRemainingMonthlyInvitations {
    fn from(value: RemainingMonthlyInvitations) -> Self {
        Self {
            Available: value.Available,
            Used: value.Used,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmInviteClient(InviteClient);

impl From<InviteClient> for WasmInviteClient {
    fn from(value: InviteClient) -> Self {
        Self(value)
    }
}

#[wasm_bindgen]
impl WasmInviteClient {
    #[wasm_bindgen(js_name = "sendNewcomerInvite")]
    pub async fn send_newcomer_invite(&self, invitee_email: String, inviter_address_id: String) -> Result<(), JsValue> {
        self.0
            .send_newcomer_invite(invitee_email, inviter_address_id)
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "checkInviteStatus")]
    pub async fn check_invite_status(
        &self,
        invitee_email: String,
        invite_notification_type: WasmInviteNotificationType,
        inviter_address_id: String,
    ) -> Result<u8, JsValue> {
        self.0
            .check_invite_status(invitee_email, invite_notification_type.into(), inviter_address_id)
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "sendEmailIntegrationInvite")]
    pub async fn send_email_integration_invite(
        &self,
        invitee_email: String,
        inviter_address_id: String,
    ) -> Result<(), JsValue> {
        self.0
            .send_email_integration_invite(invitee_email, inviter_address_id)
            .await
            .map_err(|e| e.to_js_error())
    }

    #[wasm_bindgen(js_name = "getRemainingMonthlyInvitation")]
    pub async fn get_remaining_monthly_invitation(&self) -> Result<WasmRemainingMonthlyInvitations, JsValue> {
        self.0
            .get_remaining_monthly_invitation()
            .await
            .map_err(|e| e.to_js_error())
            .map(|i| i.into())
    }
}
