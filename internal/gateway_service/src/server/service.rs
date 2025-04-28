use crate::auth::{
    auth_service_client::AuthServiceClient, // <-- КЛИЕНТ
    GenerateTokensRequest, GenerateTokensResponse,
    LoginRequest, LoginResponse,
    LogoutRequest, LogoutResponse,
    RefreshRequest, RefreshResponse,
    RegisterRequest, RegisterResponse,
    ValidateRequest, ValidateResponse,
};
use tonic::transport::Channel;
use tonic::{Request, Response};

#[derive(Clone)]
pub struct GatewayServer {
    client: AuthServiceClient<Channel>,
}

impl GatewayServer {
    pub async fn new(auth_service_addr: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = AuthServiceClient::connect(auth_service_addr).await?;
        Ok(Self { client })
    }

    pub async fn login(&mut self, req: LoginRequest) -> Result<LoginResponse, Box<dyn std::error::Error>> {
        let response = self.client.login(Request::new(req)).await?.into_inner();
        Ok(response)
    }

    pub async fn refresh(&mut self, req: RefreshRequest) -> Result<RefreshResponse, Box<dyn std::error::Error>> {
        let response = self.client.refresh(Request::new(req)).await?.into_inner();
        Ok(response)
    }

    pub async fn validate(&mut self, req: ValidateRequest) -> Result<ValidateResponse, Box<dyn std::error::Error>> {
        let response = self.client.validate(Request::new(req)).await?.into_inner();
        Ok(response)
    }

    pub async fn logout(&mut self, req: LogoutRequest) -> Result<LogoutResponse, Box<dyn std::error::Error>> {
        let response = self.client.logout(Request::new(req)).await?.into_inner();
        Ok(response)
    }

    pub async fn register(&mut self, req: RegisterRequest) -> Result<RegisterResponse, Box<dyn std::error::Error>> {
        let response = self.client.register(Request::new(req)).await?.into_inner();
        Ok(response)
    }

    pub async fn generate_tokens(&mut self, req: GenerateTokensRequest) -> Result<GenerateTokensResponse, Box<dyn std::error::Error>> {
        let response = self.client.generate_tokens(Request::new(req)).await?.into_inner();
        Ok(response)
    }
}
