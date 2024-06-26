use database::{organization::Organization, Database};
use rocket::{get, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Retrieve the organization informations from its unique identifier
#[openapi(tag = "Organizations")]
#[get("/<id>")] // <- route attribute
pub async fn from_id(
    user_data: UserData,
    database: &State<Database>,
    id: String,
) -> Custom<Result<Json<Organization>, Json<RequestError>>> {

    match database.organization_manager.from_id(&id).await {
        Ok(organization) if organization.is_some() => {
            Custom(Status::Ok, Ok(Json(organization.unwrap())))
        }
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("Organization not found with id: {id}"),
            ))
            .into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{organization::Organization, Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/organization/{}", test_org.unique_id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let org = response.into_json::<Organization>().await.unwrap();
            assert_eq!(org.unique_id, test_org.unique_id);
            assert_eq!(org.owner_id, test_org.owner_id);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_from_unknown_id() {
        run_test(|client| async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap())
                    .await;
            let request_token = request_user.get_token().unwrap();
            let id = "NO_ID";

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/organization/{id}"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let request_error = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(request_error.code, 404);
            assert_eq!(
                request_error.message,
                format!("Organization not found with id: {id}")
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/organization/{}", test_org.unique_id),
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
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/organization/{}", test_org.unique_id),
                None,
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
        })
        .await;
    }
}
