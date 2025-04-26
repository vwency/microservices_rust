use crate::auth::{
    auth_service_server::{AuthService, AuthServiceServer}, // Импортируем AuthServiceServer
    GenerateTokensRequest,
    GenerateTokensResponse,
    LoginRequest,
    LoginResponse,
    LogoutRequest,
    LogoutResponse,
    RefreshRequest,
    RefreshResponse,
    RegisterRequest,
    RegisterResponse,
    ValidateRequest,
    ValidateResponse,
};
use crate::handler::auth::AuthHandler; // Импортируем AuthHandler
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct GatewayServer {
    auth_handler: AuthHandler, // Используем AuthHandler здесь
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

pub async fn run_http3_server(
    addr: std::net::SocketAddr,
    gateway: GatewayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let auth_service = AuthServiceServer::new(gateway); // Создаем сервер с помощью AuthServiceServer

    // Здесь должна быть ваша реализация HTTP/3 сервера
    // с добавлением auth_service

    Ok(())
}
