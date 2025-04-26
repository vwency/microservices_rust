use crate::auth::{
    auth_service_server::AuthService, GenerateTokensRequest, GenerateTokensResponse, LoginRequest,
    LoginResponse, LogoutRequest, LogoutResponse, RefreshRequest, RefreshResponse, RegisterRequest,
    RegisterResponse, ValidateRequest, ValidateResponse,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct AuthHandler;

#[tonic::async_trait]
impl AuthService for AuthHandler {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        Ok(Response::new(LoginResponse {
            access_token: "example_access_token".to_string(),
            refresh_token: "example_refresh_token".to_string(),
            expires_at: 1893456000,
        }))
    }

    async fn refresh(
        &self,
        request: Request<RefreshRequest>,
    ) -> Result<Response<RefreshResponse>, Status> {
        let req = request.into_inner();

        Ok(Response::new(RefreshResponse {
            access_token: "example_new_access_token".to_string(),
            refresh_token: "example_new_refresh_token".to_string(),
            expires_at: 1893456000,
        }))
    }

    async fn validate(
        &self,
        request: Request<ValidateRequest>,
    ) -> Result<Response<ValidateResponse>, Status> {
        let req = request.into_inner();

        Ok(Response::new(ValidateResponse {
            valid: true,
            user_id: "example_user_id".to_string(),
            roles: vec!["user".to_string()],
            expires_at: 1893456000,
        }))
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();

        Ok(Response::new(LogoutResponse {
            success: true,
            message: "Successfully logged out".to_string(),
        }))
    }

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        Ok(Response::new(RegisterResponse {
            access_token: "example_registration_token".to_string(),
            refresh_token: "example_registration_refresh_token".to_string(),
            expires_at: 1893456000,
        }))
    }

    async fn generate_tokens(
        &self,
        request: Request<GenerateTokensRequest>,
    ) -> Result<Response<GenerateTokensResponse>, Status> {
        let req = request.into_inner();

        Ok(Response::new(GenerateTokensResponse {
            access_token: "example_generated_access_token".to_string(),
            refresh_token: "example_generated_refresh_token".to_string(),
            expires_at: 1893456000,
        }))
    }
}
