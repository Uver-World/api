mod api;
mod server;

pub mod cors;
pub mod model;
pub mod route;
pub mod settings;

pub use api::*;
use reqwest::{Error, Response};
use rocket::serde::DeserializeOwned;
pub use server::*;

pub async fn parse_request<T: DeserializeOwned>(
    res: Result<Response, Error>,
    ok_response: String,
) -> Result<T, String> {
    match res {
        Ok(res) if res.status().is_success() => {
            let json = res.json::<T>().await;
            match json {
                Ok(json) => Ok(json),
                _ => Err("json issue error".to_string()),
            }
        }
        Ok(res) => Err(format!("[{ok_response}]\nlog={res:#?}")),
        Err(err) => Err(format!("reqwest error: {err}")),
    }
}
