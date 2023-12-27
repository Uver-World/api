use database::Database;
use opentelemetry::{global, KeyValue, trace::{Tracer, Span}};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Request, Response};
use rocket::*;

use crate::api_telemetry;

pub struct TelemetryFairing;

#[rocket::async_trait]
impl Fairing for TelemetryFairing {
    fn info(&self) -> Info {
        Info {
            name: "Logging requests to SigNoz",
            kind: Kind::Response | Kind::Liftoff,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        
        let tracer = global::tracer("api");
        
        let mut span = tracer.start("request");

        span.set_attribute(KeyValue::new("request", format!("{:#}", request)));
        span.set_attribute(KeyValue::new("response", format!("{:?}", response)));
    }
    
    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        // Retrieve the State<Database> and pass it to the telemetry start function
        if let Some(database) = rocket.state::<Database>() {
            api_telemetry::start(database.clone());
        } else {
            eprintln!("Database state not found");
        }
    }

}
