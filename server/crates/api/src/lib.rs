#![feature(coverage_attribute)]
#![feature(let_chains)]
#![feature(async_closure)]

mod api;
mod server;

pub mod cors;
pub mod model;
pub mod route;
pub mod settings;
pub mod testing;

pub use crate::{api::*, server::*};

use rocket::response::status::Custom;
use rocket_okapi::okapi::schemars;
use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};

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
