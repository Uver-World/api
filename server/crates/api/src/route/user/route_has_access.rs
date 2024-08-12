use database::{Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{user_id::UserId, user_token::UserData},
    RequestError,
};

#[openapi(tag = "Users")]
#[post("/has_access", data = "<user_id>", format = "application/json")] // <- route attribute
pub async fn has_access(
    user_data: UserData,
    database: &State<Database>,
    user_id: Json<UserId>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {

    let server_id = user_data.id.unwrap();
    let user_id = user_id.0;
    match database.user_manager.from_id(&server_id).await {
        Ok(user) if user.is_some() => {
            match database
                .organization_manager
                .has_access_to_server(&server_id, &user_id.0)
                .await
            {
                Ok(result) => Custom(Status::Ok, Ok(Json(result))),
                Err(_) => Custom(
                    Status::Ok,
                    Err(RequestError::from(Custom(
                        Status::InternalServerError,
                        format!("A database error occured."),
                    ))
                    .into()),
                ),
            }
        }
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("User not found with id: {}", user_id.0),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{Database};
    use rocket::http::{Method, Status};

    use crate::{
        model::user_id::UserId,
        testing::{self, dispatch_request, run_test},
    };

    #[rocket::async_test]
    async fn test_has_access() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_server = testing::get_user(database).await;
            let test_user = testing::get_user(database).await;
            let _test_org =
                testing::create_org(database, &test_user, vec![test_server.unique_id.clone()])
                    .await;

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/has_access"),
                Some(serde_json::to_string(&UserId(test_user.unique_id)).unwrap()),
                Some(test_server.get_token().unwrap().to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert!(response);
        })
        .await;
    }
}
