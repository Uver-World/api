use database::user::User;
use rocket::{fairing::AdHoc, *};

use database::*;
use rocket_okapi::mount_endpoints_and_merged_docs;
use rocket_okapi::rapidoc::*;
use rocket_okapi::settings::UrlObject;

use telemetry::TelemetrySettings;


use crate::api_telemetry;
use crate::settings::ApiSettings;
use crate::{cors::CORS, route::ApiRoute, Server};

async fn create_default_website_user(database: &Database) {
    let website_missing_response = database.user_manager.website_missing();
    if website_missing_response.await.unwrap() {
        println!("NO USER WITH THE WEBSITE GROUP EXISTS");
        let unique_id: u64 = Server::generate_unique_id();
        let timestamp = Server::current_time();
        let _ = database.user_manager.create_user(&User::default_website_user(unique_id.to_string(), timestamp)).await;
        println!("A NEW USER WITH THE WEBSITE GROUP HAS BEEN CREATED, PLEASE CHECK DATABASE");
    }
}

fn init_telemetry(settings: TelemetrySettings) -> AdHoc {
        AdHoc::on_ignite("Launching telemetry", |rocket| async {
           telemetry::start_telemetry(settings);
            api_telemetry::start();
            rocket
        })
}

fn init_db(settings: DatabaseSettings) -> AdHoc {
    AdHoc::on_ignite("Connecting to MongoDB", |rocket| async {
        match Database::init(&settings).await {
            Ok(database) => {
                create_default_website_user(&database).await;
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
    };
    rocket_builder.manage(Server::default())
}
