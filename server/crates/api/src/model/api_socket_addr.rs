use std::net::SocketAddr;

use rocket::request::{self, FromRequest, Outcome, Request};
use rocket_okapi::{
    gen::OpenApiGenerator,
    request::{OpenApiFromRequest, RequestHeaderInput},
};

pub struct ApiSocketAddr(pub SocketAddr);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiSocketAddr {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request
            .remote()
            .map(|addr| Outcome::Success(ApiSocketAddr(addr.clone())))
            .unwrap_or_else(|| Outcome::Success(ApiSocketAddr("127.0.0.1:0".parse().unwrap())))
    }
}

impl<'a> OpenApiFromRequest<'a> for ApiSocketAddr {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(RequestHeaderInput::None)
    }
}
