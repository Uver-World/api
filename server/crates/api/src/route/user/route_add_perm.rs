use database::{Database};
use rocket::{post, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

// A route to add a permission to a user.
#[openapi(tag = "Users")]
#[post("/<user_id>/permissions/<permission_id>")]
pub async fn add_perm(
  user_data: UserData,
    database: &State<Database>,
    user_id: String,
    permission_id: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if user_data.id.is_none() {
        return Custom(
            Status::Unauthorized,
            Err(RequestError::from(Custom(
                Status::Unauthorized,
                format!("Token is invalid"),
            ))
            .into()),
        );
    }

    if !database
        .user_manager
        .user_exists(&user_id)
        .await
    {
        return Custom(
            Status::NotFound,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("User not found"),
            ))
            .into()),
        );
    }

    if !database
        .permission_manager
        .permission_exists(&permission_id)
        .await
    {
        return Custom(
            Status::NotFound,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("Permission not found"),
            ))
            .into()),
        );
    }

    let permission_add = match database
        .permission_manager
        .get_permission_id("permission.add")
        .await
    {
        Ok(id) => id,
        Err(_) => {
            return Custom(
                Status::NotFound,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Unknown permission"),
                ))
                .into()),
            )
        }
    };

    if !database.user_manager.has_permission(&user_data.id.unwrap(), &permission_add).await {
        return Custom(
            Status::Forbidden,
            Err(RequestError::from(Custom(
                Status::Forbidden,
                format!("Permission denied"),
            ))
            .into()),
        );
    }

    let permission_id_clone = permission_id.clone();
    
    match database
        .user_manager
        .add_permission(user_id, permission_id_clone)
        .await
    {
        Ok(_) => Custom(Status::Created, Ok(Json(true))),
        Err(err) => Custom(
          Status::Ok,
          Err(RequestError::from(Custom(
              Status::NotFound,
              format!("User not found"),
          ))
          .into()),
      ),
    }
}