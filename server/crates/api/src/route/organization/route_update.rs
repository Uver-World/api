use database::{organization::OrganizationUpdate, Database};
use rocket::{http::Status, patch, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Update the organization informations from its id
#[openapi(tag = "Organizations")]
#[patch("/<id>", data = "<organization_update>", format = "application/json")] // <- route attribute
pub async fn update(
    _user_data: UserData,
    database: &State<Database>,
    id: String,
    organization_update: Json<Vec<OrganizationUpdate>>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {


    match database
        .organization_manager
        .update_organization(&id, organization_update.0)
        .await
    {
        Ok(result) if result.matched_count > 0 => Custom(Status::Ok, Ok(Json(true))),
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
            Err(RequestError::from(Custom(Status::InternalServerError, err.to_string())).into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{
        organization::{Organization, OrganizationUpdate},
        Database,
    };
    use rocket::http::{Method, Status};

    use crate::{
        testing::{self, dispatch_request, run_test},
        RequestError,
    };

    #[rocket::async_test]
    async fn test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;
            let updates = vec![
                OrganizationUpdate::Name("Another name".to_string()),
                OrganizationUpdate::OwnerId("SOME_OTHER_ID".to_string()),
            ];

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/organization/{}", test_org.unique_id),
                Some(serde_json::to_string(&updates).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<bool>().await.unwrap();
            assert_eq!(response, true);
            let updated_org = database
                .organization_manager
                .from_id(&test_org.unique_id)
                .await
                .unwrap()
                .unwrap();

            check_org_difference(false, &test_org, &updated_org)
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknown_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let updates = vec![
                OrganizationUpdate::Name("Another name".to_string()),
                OrganizationUpdate::OwnerId("SOME_OTHER_ID".to_string()),
            ];

            let id = "NO_ID";
            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/organization/{id}"),
                Some(serde_json::to_string(&updates).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let response = response.into_json::<RequestError>().await.unwrap();
            assert_eq!(response.code, 404);
            assert_eq!(
                response.message,
                format!("Organization not found with id: {id}")
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_update() {
        _unauthorized_test_update().await;
        _unauthorized_test_update().await;
    }

    async fn _unauthorized_test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;
            let updates = vec![
                OrganizationUpdate::Name("Another name".to_string()),
                OrganizationUpdate::OwnerId("SOME_OTHER_ID".to_string()),
            ];

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/organization/{}", test_org.unique_id),
                Some(serde_json::to_string(&updates).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
            let updated_org = database
                .organization_manager
                .from_id(&test_org.unique_id)
                .await
                .unwrap()
                .unwrap();

            check_org_difference(true, &test_org, &updated_org)
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_update() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let test_org = testing::get_org(database, &test_user).await;
            let updates = vec![
                OrganizationUpdate::Name("Another name".to_string()),
                OrganizationUpdate::OwnerId("SOME_OTHER_ID".to_string()),
            ];

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/organization/{}", test_org.unique_id),
                Some(serde_json::to_string(&updates).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
            let updated_org = database
                .organization_manager
                .from_id(&test_org.unique_id)
                .await
                .unwrap()
                .unwrap();

            check_org_difference(true, &test_org, &updated_org)
        })
        .await;
    }

    /// Check if the organization has changed between the request and after it.
    /// If is_same is set false, then it will assert_ne! instead of asserting equal.
    fn check_org_difference(is_same: bool, org1: &Organization, org2: &Organization) {
        // first we're checking if they're of the same id.
        assert_eq!(org1.unique_id, org2.unique_id);

        if is_same {
            assert_eq!(org1.name, org2.name);
            assert_eq!(org1.owner_id, org2.owner_id);
        } else {
            assert_ne!(org1.name, org2.name);
            assert_ne!(org1.owner_id, org2.owner_id);
        }
    }
}
