use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct UserSettings {
    pub TwoFactorAuth: TwoFactorAuth, // TODO:: use proton-api-core::domain::user_settings
                                      // pub email: UserSettingsEmail,
                                      // pub password: UserSettingsPassword,
                                      // pub phone: UserSettingsPhone,
                                      // #[serde(rename = "2FA")]
                                      // pub two_factor_auth: UserSettings2FA,
                                      // pub news: u32,
                                      // pub locale: String,
                                      // pub log_auth: UserLogAuth,
                                      // pub invoice_text: String,
                                      // pub density: UserSettingsDensity,
                                      // pub week_start: UserSettingsWeekStart,
                                      // pub date_format: UserSettingsDateFormat,
                                      // pub time_format: UserSettingsTimeFormat,
                                      // pub welcome: ProtonBoolean,
                                      // pub early_access: ProtonBoolean,
                                      // pub flags: UserSettingsFlags,
                                      // pub referral: Option<UserSettingsReferral>,
                                      // pub device_recovery: ProtonBoolean,
                                      // pub telemetry: ProtonBoolean,
                                      // pub crash_reports: ProtonBoolean,
                                      // pub hide_side_panel: ProtonBoolean,
                                      // pub high_security: UserSettingsHighSecurity,
                                      // pub session_account_recovery: ProtonBoolean,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TwoFactorAuth {
    pub Enabled: u32,
    pub Allowed: u32,
    pub ExpirationTime: u32,
}
