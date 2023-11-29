use database::{group::Group, Database, organization::Organization};
use rocket::{http::Status, get, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::user_token::UserData,
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

#[cfg(test)]
mod tests {

    use database::{group::Group, Database, organization::Organization};
    use rocket::http::{Method, Status};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_get_organizations() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::User).await;
            let request_user = testing::get_user(database, Group::User).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Get,
                format!("/user/id/{}/organizations", test_user.unique_id),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let organizations = response.into_json::<Vec<Organization>>().await.unwrap();
            assert_eq!(organizations.len(), 0);
        });
    }
}