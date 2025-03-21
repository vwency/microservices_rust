use tonic::{Request, Response, Status};
use crate::hello::hello_service_server::HelloService;
use crate::hello::{HelloRequest, HelloResponse};

#[derive(Debug, Default)]
pub struct MyHelloService {}

#[tonic::async_trait]
impl HelloService for MyHelloService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        let name = request.into_inner().name;

        if name.to_lowercase().contains("hello") {
            let reply = HelloResponse {
                message: format!("Hello, {}!", name),
            };
            Ok(Response::new(reply))
        } else {
            Err(Status::invalid_argument("Message must contain 'hello'"))
        }
    }
}
