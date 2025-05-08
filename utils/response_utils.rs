use bytes::Bytes;
use http_body_util::{combinators::BoxBody, Full};
use hyper::{Response, StatusCode};
use serde::Serialize;

pub type HttpBody = BoxBody<Bytes, hyper::Error>;
pub type HttpResponse = Response<HttpBody>;

pub fn json_response<T: Serialize>(
    value: &T,
) -> Result<HttpResponse, Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_vec(value)?;
    let body = Full::new(Bytes::from(json)).map_err(|_| hyper::Error::new_incomplete_message());
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(BoxBody::new(body))
        .unwrap())
}

pub fn error_response(
    status: StatusCode,
    message: &str,
) -> HttpResponse {
    let body = Full::new(Bytes::from(message.to_string())).map_err(|_| hyper::Error::new_incomplete_message());
    Response::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(BoxBody::new(body))
        .unwrap()
}
