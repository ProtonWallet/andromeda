#[cfg(feature = "mocking")]
pub mod mock_utils {
    use mockall::mock;

    use crate::{
        error::Error,
        proton_users::{
            GetAuthInfoRequest, GetAuthInfoResponseBody, GetAuthModulusResponse, ProtonSrpClientProofs, ProtonUser,
            ProtonUserSettings, ProtonUsersClientExt,
        },
    };

    mock! {
        pub ProtonUsersClient {}

        #[async_trait::async_trait]
        impl ProtonUsersClientExt for ProtonUsersClient {
            async fn get_auth_modulus(&self) -> Result<GetAuthModulusResponse, Error>;

            async fn get_auth_info(&self, req: GetAuthInfoRequest) -> Result<GetAuthInfoResponseBody, Error>;

            async fn unlock_password_change(&self, proofs: ProtonSrpClientProofs) -> Result<String, Error>;

            async fn unlock_sensitive_settings(&self, proofs: ProtonSrpClientProofs) -> Result<String, Error>;

            async fn lock_sensitive_settings(&self) -> Result<u32, Error>;

            async fn get_user_info(&self) -> Result<ProtonUser, Error>;

            async fn get_user_settings(&self) -> Result<ProtonUserSettings, Error>;
        }
    }
}
