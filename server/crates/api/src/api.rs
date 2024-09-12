
use rocket::{fairing::AdHoc, *};

use database::*;
use rocket_okapi::mount_endpoints_and_merged_docs;
use rocket_okapi::rapidoc::*;
use rocket_okapi::settings::UrlObject;

use telemetry::TelemetrySettings;

use crate::api_telemetry::TelemetryFairing;
use crate::settings::ApiSettings;
use crate::{cors::CORS, route::ApiRoute, Server};

fn init_telemetry(settings: TelemetrySettings) -> AdHoc {
        AdHoc::on_ignite("Launching telemetry", |rocket| async {
           telemetry::start_telemetry(settings);

            rocket.attach(TelemetryFairing)
        })
}

fn init_db(settings: DatabaseSettings) -> AdHoc {
    AdHoc::on_ignite("Connecting to MongoDB", |rocket| async {
        match Database::init(&settings).await {
            Ok(database) => {
                rocket.manage(settings).manage(database)
            },
            Err(error) => {
                panic!("Cannot connect to MongoDB instance:: {error:?}");
            }
        }
    })
}

pub fn get_rocket() -> Rocket<Build> {
    let settings = ApiSettings::retrieve();
    let mut rocket_builder = Rocket::build()
        .attach(init_db(settings.database.clone()))
        .attach(init_telemetry(settings.telemetry.clone()))
        .attach(CORS)
        .manage(settings)
        .mount(
            "/rapidoc/",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("Endpoints", "../openapi.json")],
                    ..Default::default()
                },
                hide_show: HideShowConfig {
                    allow_spec_url_load: false,
                    allow_spec_file_load: false,
                    ..Default::default()
                },
                ..Default::default()
            }),
        );

    let openapi_settings = rocket_okapi::settings::OpenApiSettings::default();
    mount_endpoints_and_merged_docs! {
        rocket_builder, "/".to_owned(), openapi_settings,
        "/" => ApiRoute::Root.retrieve_routes(),
        "/user" => ApiRoute::User.retrieve_routes(),
        "/organization" => ApiRoute::Organization.retrieve_routes(),
        "/asset" => ApiRoute::Asset.retrieve_routes(),
    };
    rocket_builder.manage(Server::default())
}
