use database::{project::ProjectUpdateData, Database};
use rocket::{http::Status, patch, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{model::user_token::UserData, RequestError};

/// Update the project informations from its id
#[openapi(tag = "Organizations")]
#[patch("/<id>/projects", data = "<project_update>", format = "application/json")] // <- route attribute
pub async fn update_project(
    user_data: UserData,
    database: &State<Database>,
    id: String,
    project_update: Json<ProjectUpdateData>,
) -> Custom<Result<Json<bool>, Json<RequestError>>> {


    // If organization not found 
    match database
        .organization_manager
        .from_id(id.as_str())
        .await
    {
        Ok(Some(_)) => (),
        Ok(None) => {
            return Custom(
                Status::Ok,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Organization not found with id: {id}"),
                ))
                .into()),
            )
        }
        Err(err) => {
            return Custom(
                Status::Ok,
                Err(RequestError::from(Custom(Status::InternalServerError, err.to_string())).into()),
            )
        }
    }

    // if project not found
    match database
        .project_manager
        .from_id(project_update.0.project_id.as_str())
        .await
    {
        Ok(Some(_)) => (),
        Ok(None) => {
            return Custom(
                Status::NotFound,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    format!("Project not found with id: {id}", id = project_update.0.project_id),
                ))
                .into()),
            )
        }
        Err(err) => {
            return Custom(
                Status::Ok,
                Err(RequestError::from(Custom(Status::InternalServerError, err.to_string())).into()),
            )
        }
    }

    match database
        .project_manager
        .update_project(&project_update.0.project_id, project_update.0.project_update)
        .await
    {
        Ok(_)=> Custom(Status::Ok, Ok(Json(true))),
        Err(err) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::InternalServerError,
                err.to_string(),
            ))
            .into()),
        ),
    }

}

#[cfg(test)]
mod tests {

    use database::{
        project::ProjectUpdateData,
        Database,
    };
    use rocket::http::{Method, Status};

    use crate::testing::{self, dispatch_request, run_test};

    #[rocket::async_test]
    async fn test_unknow_organization() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let updates = ProjectUpdateData {
                project_id: "cdcdgr".to_string(),
                project_update: vec![],
            };

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/organization/{}/projects", test_user.unique_id),
                Some(serde_json::to_string(&updates).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_unknow_project() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database).await;
            let request_user = testing::get_user(database).await;
            let request_token = request_user.get_token().unwrap();
            let test_org = testing::get_org(database, &test_user).await;
            let updates = ProjectUpdateData {
                project_id: "cdcdgr".to_string(),
                project_update: vec![],
            };

            let response = dispatch_request(
                &client,
                Method::Patch,
                format!("/organization/{}/projects", test_org.unique_id),
                Some(serde_json::to_string(&updates).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            println!("{:?}", response.status());

            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }
}