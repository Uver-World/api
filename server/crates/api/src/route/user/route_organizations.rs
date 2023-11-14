use database::{group::Group, Database, organization::Organization};
use rocket::{http::Status, get, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{user_token::UserData},
    RequestError,
};

#[openapi(tag = "Users")]
#[get("/id/<user_id>/organizations")]
pub async fn get_organizations(
    user_data: UserData,
    database: &State<Database>,
    user_id: String,
) -> Custom<Result<Json<Vec<Organization>>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::User]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    match database
        .organization_manager
        .get_organizations_from_user(&user_id)
        .await
    {
        Ok(result) => Custom(Status::Ok, Ok(Json(result))),
        Err(_) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                format!("A database error occurred."),
            ))
            .into()),
        ),
    }
}