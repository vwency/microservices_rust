pub mod service;

pub mod generated {
    tonic::include_proto!("auth_service"); // это будет генерировать структуру для `AuthService` и всех его методов
}
