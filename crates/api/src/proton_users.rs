use std::sync::Arc;

use muon::rest::core::v4::{keys::salts::KeySalt, users::User};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ChildSession {
    pub session_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub scopes: Vec<String>,
}
pub struct UserData {
    pub user: User,
    pub key_salts: Vec<KeySalt>,
}

use crate::{
    core::{ApiClient, ProtonResponseExt},
    error::Error,
    ProtonWalletApiClient, BASE_CORE_API_V4,
};

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonUserSettingsResponse {
    Code: u32,
    UserSettings: ProtonUserSettings,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProtonUserSettings {
    Email: EmailSettings,
    Password: Option<PasswordSettings>,
    Phone: Option<PhoneSettings>,
    #[serde(rename = "2FA")]
    two_fa: Option<TwoFASettings>,
    News: u32,
    Locale: String,
    LogAuth: u32,
    InvoiceText: String,
    Density: u32,
    WeekStart: u32,
    DateFormat: u32,
    TimeFormat: u32,
    Welcome: u32,
    WelcomeFlag: u32,
    EarlyAccess: u32,
    Flags: Option<FlagsSettings>,
    Referral: Option<ReferralSettings>,
    DeviceRecovery: u32,
    Telemetry: u32,
    CrashReports: u32,
    HideSidePanel: u32,
    HighSecurity: HighSecuritySettings,
    SessionAccountRecovery: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct EmailSettings {
    Value: String,
    Status: u32,
    Notify: u32,
    Reset: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PasswordSettings {
    // PasswordSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct PhoneSettings {
    // PhoneSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct TwoFASettings {
    Enabled: u32,
    Allowed: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct FlagsSettings {
    // FlagsSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ReferralSettings {
    // ReferralSettings is empty here we need it in the parser but we don't use it yet in our implementation
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct HighSecuritySettings {
    Eligible: u32,
    Value: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiProtonUserResponse {
    Code: u32,
    User: ProtonUser,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProtonUser {
    ID: String,
    Name: String,
    UsedSpace: u64,
    Currency: String,
    Credit: u32,
    CreateTime: u64,
    MaxSpace: u64,
    MaxUpload: u64,
    Role: u32,
    Private: u32,
    Subscribed: u32,
    Services: u32,
    Delinquent: u32,
    OrganizationPrivateKey: Option<String>,
    Email: String,
    DisplayName: String,
    Keys: Option<Vec<ProtonUserKey>>,
}
#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ProtonUserKey {
    ID: String,
    Version: u32,
    PrivateKey: String,
    RecoverySecret: Option<String>,
    RecoverySecretSignature: Option<String>,
    Token: Option<String>,
    Fingerprint: String,
    Primary: u32,
    Active: u32,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct ApiUserInfo {
    pub ID: String,
    pub Name: String,
    pub Email: String,
    pub CanonicalEmail: String,
    pub IsProton: u32,
}

#[derive(Clone)]
pub struct ProtonUsersClient {
    api_client: Arc<ProtonWalletApiClient>,
}

impl ApiClient for ProtonUsersClient {
    fn new(api_client: Arc<ProtonWalletApiClient>) -> Self {
        Self { api_client }
    }

    fn api_client(&self) -> &Arc<ProtonWalletApiClient> {
        return &self.api_client;
    }

    fn base_url(&self) -> &str {
        BASE_CORE_API_V4
    }
}

impl ProtonUsersClient {
    // get proton user info. This includes the user's keys.
    pub async fn get_user_info(&self) -> Result<ProtonUser, Error> {
        let request = self.get("users");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<ApiProtonUserResponse>()?;
        Ok(parsed.User)
    }

    // get proton user settings.
    //  used for 2fa settings and password recovery etc..
    pub async fn get_user_settings(&self) -> Result<ProtonUserSettings, Error> {
        let request = self.get("settings");

        let response = self.api_client.send(request).await?;
        let parsed = response.parse_response::<ApiProtonUserSettingsResponse>()?;
        Ok(parsed.UserSettings)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::ProtonUsersClient;
    use crate::{
        core::ApiClient,
        tests::utils::{common_api_client, setup_test_connection},
        BASE_CORE_API_V4,
    };

    #[tokio::test]
    #[ignore]
    async fn should_get_user_info() {
        let api_client = common_api_client().await;
        let client = ProtonUsersClient::new(api_client);
        let res = client.get_user_info().await;
        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }
    #[tokio::test]
    #[ignore]
    async fn should_get_user_settings() {
        let api_client = common_api_client().await;
        let client = ProtonUsersClient::new(api_client);
        let res = client.get_user_settings().await;
        println!("request done: {:?}", res);
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_user_info_code_1000() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "User": {
                    "ID": "MJLke8kWh1BBvG95JBIrZvzpgsZ94hNNgjNHVyhXMiv4g9cn6SgvqiIFR5cigpml2LD_iUk_3DkV29oojTt3eA==",
                    "Name": "abc",
                    "UsedSpace": 96691332,
                    "Currency": "USD",
                    "Credit": 0,
                    "CreateTime": 1654615960,
                    "MaxSpace": 10737418240i64,
                    "MaxUpload": 26214400i64,
                    "Role": 2,
                    "Private": 1,
                    "Subscribed": 1,
                    "Services": 1,
                    "Delinquent": 0,
                    "OrganizationPrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK-----*",
                    "Email": "abc@protonmail.ch",
                    "DisplayName": "abc",
                    "Keys": [
                        {
                            "ID": "IlnTbqicN-2HfUGIn-ki8bqZfLqNj5ErUB0z24Qx5g-4NvrrIc6GLvEpj2EPfwGDv28aKYVRRrSgEFhR_zhlkA==",
                            "Version": 3,
                            "PrivateKey": "-----BEGIN PGP PRIVATE KEY BLOCK-----*-----END PGP PRIVATE KEY BLOCK-----",
                            "Token": "-----BEGIN PGP MESSAGE-----.*-----END PGP MESSAGE-----",
                            "Fingerprint": "c93f767df53b0ca8395cfde90483475164ec6353",
                            "Primary": 1,
                            "Active": 1,
                        }
                    ]
                },
                "Code": 1000
            }
        );
        let req_path: String = format!("{}/users", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_info = users_client.get_user_info().await;
        match user_info {
            Ok(value) => {
                assert!(value.DisplayName == "abc");
                assert!(value.Name == "abc");
                assert!(value.Keys.unwrap().len() > 0);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Expected Ok variant but got Err")
            }
        }
    }

    #[tokio::test]
    async fn test_get_user_info_deserialize_error() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({});
        let req_path: String = format!("{}/users", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(400).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_info = users_client.get_user_info().await;
        assert!(user_info.is_err());
    }

    #[tokio::test]
    async fn test_get_users_settings_code_1000() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!(
            {
                "Code": 1000,
                "UserSettings": {
                  "Email": {
                    "Value": "abc@gmail.com",
                    "Status": 0,
                    "Notify": 1,
                    "Reset": 0
                  },
                  "Password": {
                    "Mode": 2,
                    "ExpirationTime": null
                  },
                  "Phone": {
                    "Value": "+18005555555",
                    "Status": 0,
                    "Notify": 0,
                    "Reset": 0
                  },
                  "2FA": {
                    "Enabled": 3,
                    "Allowed": 3,
                    "ExpirationTime": null,
                    "U2FKeys": [
                      {
                        "Label": "A name",
                        "KeyHandle": "aKeyHandle",
                        "Compromised": 0
                      }
                    ],
                    "RegisteredKeys": [
                      {
                        "AttestationFormat": "fido2-u2f",
                        "CredentialID": [
                          null
                        ],
                        "Name": "My security key"
                      }
                    ]
                  },
                  "News": 244,
                  "Locale": "en_US",
                  "LogAuth": 2,
                  "InvoiceText": "रिवार में हुआ। ज檷\\n Cartoon Law Services\\n 1 DisneyWorld Lane\\n Orlando, FL, 12345\\n VAT",
                  "Density": 0,
                  "Theme": {},
                  "ThemeType": 1,
                  "WeekStart": 1,
                  "DateFormat": 1,
                  "TimeFormat": 1,
                  "Welcome": 1,
                  "WelcomeFlag": 1,
                  "EarlyAccess": 1,
                  "Flags": {
                    "Welcomed": 0,
                    "InAppPromosHidden": 0
                  },
                  "Referral": {
                    "Link": "https://pr.tn/ref/ERBYvlX8SC4KOyb",
                    "Eligible": true
                  },
                  "DeviceRecovery": 1,
                  "Telemetry": 1,
                  "CrashReports": 1,
                  "HideSidePanel": 1,
                  "HighSecurity": {
                    "Eligible": 1,
                    "Value": 1
                  },
                  "SessionAccountRecovery": 1
                }
              }
        );
        let req_path: String = format!("{}/settings", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(200).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_settings = users_client.get_user_settings().await;
        match user_settings {
            Ok(value) => {
                assert!(value.HideSidePanel == 1, "Expected hide_side_panel to be 1.");
                assert!(value.Locale == "en_US", "Expected locale to be en_US.");
                assert!(value.News == 244, "Expected news to be 244.");
                assert!(value.Phone.is_some(), "Expected phone to be Some.");
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Expected Ok variant but got Err")
            }
        }
    }

    #[tokio::test]
    async fn test_get_users_settings_deserialize_error() {
        let mock_server = MockServer::start().await;
        let response_body = serde_json::json!({});
        let req_path: String = format!("{}/settings", BASE_CORE_API_V4);
        let response = ResponseTemplate::new(400).set_body_json(response_body);
        Mock::given(method("GET"))
            .and(path(req_path))
            .respond_with(response)
            .mount(&mock_server)
            .await;
        let api_client = setup_test_connection(mock_server.uri());
        let users_client = ProtonUsersClient::new(api_client);
        let user_settings = users_client.get_user_settings().await;
        assert!(user_settings.is_err());
    }
}
