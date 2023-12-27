use opentelemetry::{global, KeyValue, trace::{Tracer, Span}};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};

use rocket::*;
use rocket_okapi::openapi;

pub struct TelemetryFairing;

#[rocket::async_trait]
impl Fairing for TelemetryFairing {
    fn info(&self) -> Info {
        Info {
            name: "Logging requests to SigNoz",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        
        let tracer = global::tracer("api");
        
        let mut span = tracer.start("request");

        span.set_attribute(KeyValue::new("request", format!("{:#}", request)));
        span.set_attribute(KeyValue::new("response", format!("{:?}", response)));
    }
}
