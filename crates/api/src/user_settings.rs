use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct UserSettings {
    pub TwoFactorAuth: TwoFactorAuth,
    // TODO:: use proton-api-core::domain::user_settings
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TwoFactorAuth {
    pub Enabled: u32,
    pub Allowed: u32,
    pub ExpirationTime: u32,
}
