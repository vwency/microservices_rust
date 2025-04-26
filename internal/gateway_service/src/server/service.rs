use crate::handler::auth::AuthHandler;
use auth_service::{
    auth_service_server::AuthService, GenerateTokensRequest, GenerateTokensResponse, LoginRequest,
    LoginResponse, LogoutRequest, LogoutResponse, RefreshRequest, RefreshResponse,
    RegisterRequest, RegisterResponse, ValidateRequest, ValidateResponse,
};
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct GatewayServer {
    auth_handler: AuthHandler,
}

#[tonic::async_trait]
impl AuthService for GatewayServer {
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        self.auth_handler.login(request).await
    }

    async fn refresh(
        &self,
        request: Request<RefreshRequest>,
    ) -> Result<Response<RefreshResponse>, Status> {
        self.auth_handler.refresh(request).await
    }

    async fn validate(
        &self,
        request: Request<ValidateRequest>,
    ) -> Result<Response<ValidateResponse>, Status> {
        self.auth_handler.validate(request).await
    }

    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        self.auth_handler.logout(request).await
    }

    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        self.auth_handler.register(request).await
    }

    async fn generate_tokens(
        &self,
        request: Request<GenerateTokensRequest>,
    ) -> Result<Response<GenerateTokensResponse>, Status> {
        self.auth_handler.generate_tokens(request).await
    }
}