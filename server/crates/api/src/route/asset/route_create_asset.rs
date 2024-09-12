use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use database::{managers::AssetManager, asset::Asset, Database};
use crate::RequestError;

#[openapi(tag = "Assets")]
#[post("/create", data = "<new_asset>", format = "application/json")]
pub async fn create_asset(
    database: &State<Database>,
    new_asset: Json<Asset>,
) -> Custom<Result<Json<String>, Json<RequestError>>> {
    let asset = new_asset.into_inner();

    match database.asset_manager.create_asset(&asset).await {
        Ok(result) => {
            let success_message = format!("Asset created with ID: {}", result.inserted_id);
            Custom(Status::Ok, Ok(Json(success_message)))
        }
        Err(err) => {
            let error_message = format!("Failed to create asset: {}", err);
            Custom(
                Status::InternalServerError,
                Err(Json(RequestError::from(Custom(
                    Status::InternalServerError,
                    error_message,
                )))),
            )
        }
    }
}
