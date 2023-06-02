use database::{group::Group, user::User, Database};
use rocket::{get, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Retrieve the user informations from its unique identifier
#[openapi(tag = "Users")]
#[get("/id/<id>")] // <- route attribute
pub async fn from_id(
    user_data: UserData,
    database: &State<Database>,
    id: String,
) -> Custom<Result<Json<User>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website, Group::Server]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    match database.user_manager.from_id(&id).await {
        Ok(user) if user.is_some() => Custom(Status::Ok, Ok(Json(user.unwrap()))),
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("User not found with id: {id}"),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{group::Group, user::User, Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let website_user = testing::get_user(database, Group::Website).await;
            let website_token = website_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/id/{}", test_user.unique_id),
                None,
                Some(website_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let user = response.into_json::<User>().await.unwrap();
            assert_eq!(user.unique_id, test_user.unique_id);
            assert_eq!(user.username, test_user.username);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_from_unknown_id() {
        run_test(|client| async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap(), Group::Website)
                    .await;
            let request_token = request_user.get_token().unwrap();
            let id = "NO_ID";

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/id/{id}"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let request_error = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(request_error.code, 404);
            assert_eq!(
                request_error.message,
                format!("User not found with id: {id}")
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, Group::User).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/id/{}", test_user.unique_id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_from_id() {
        run_test(|client| async move {
            let test_user =
                testing::get_user(client.rocket().state::<Database>().unwrap(), Group::User).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/id/{}", test_user.get_token().unwrap()),
                None,
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
        })
        .await;
    }
}
