use database::{group::Group, Database};
use rocket::{delete, http::Status, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Delete the user from its id
#[openapi(tag = "Organizations")]
#[delete("/<id>")] // <- route attribute
pub async fn delete_from_id(
    user_data: UserData,
    database: &State<Database>,
    id: String,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    match database.organization_manager.delete_organization(&id).await {
        Ok(Some(result)) if result.deleted_count > 0 => Custom(Status::Ok, Ok(Json(true))),
        Ok(_) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::NotFound,
                format!("Organization not found with id: {id}"),
            ))
            .into()),
        ),
        Err(err) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(Status::InternalServerError, err)).into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{group::Group, Database};
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_delete_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Delete,
                format!("/organization/{}", &test_org.unique_id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert!(response);
            // Organization should have been deleted
            assert!(database
                .organization_manager
                .from_id(&test_org.unique_id)
                .await
                .unwrap()
                .is_none());
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_delete_from_unknown_id() {
        run_test(|client| async move {
            let request_user =
                testing::get_user(client.rocket().state::<Database>().unwrap(), Group::Website)
                    .await;
            let request_token = request_user.get_token().unwrap();
            let id = "NO_ID";

            let response = dispatch_request(
                &client,
                Method::Delete,
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
    async fn unauthorized_test_delete_from_id() {
        _unauthorized_test_delete_from_id(Group::User).await;
        _unauthorized_test_delete_from_id(Group::Server).await;
    }

    async fn _unauthorized_test_delete_from_id(request_group: Group) {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, request_group).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Delete,
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
    async fn forbidden_test_delete_from_id() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let test_org = testing::get_org(database, &test_user).await;

            let response = dispatch_request(
                &client,
                Method::Delete,
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
