#![feature(let_chains)]

mod api;
mod server;

pub mod cors;
pub mod model;
pub mod route;
pub mod settings;
pub mod testing;

pub use crate::{api::*, server::*};

use reqwest::{Error, Response};
use rocket::response::status::Custom;
use rocket::serde::DeserializeOwned;
use rocket_okapi::okapi::schemars;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct RequestError {
    pub code: u16,
    pub message: String,
}

impl From<Custom<String>> for RequestError {
    fn from(value: Custom<String>) -> Self {
        Self {
            code: value.0.code,
            message: value.1,
        }
    }
}
