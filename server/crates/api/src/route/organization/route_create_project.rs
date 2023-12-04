use database::{group::Group, Database, project::Project};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{project_init::ProjectInit, user_token::UserData},
    RequestError, Server,
};

/// Register a new project in the organization
///
/// Requires 'Website' group
#[openapi(tag = "Organizations")]
#[post("/<id>/projects", data = "<project>", format = "application/json")] // <- route attribute
// Add new project to organization
pub async fn create_project(
    user_data: UserData,
    database: &State<Database>,
    project: Json<ProjectInit>,
    id: String,
) -> Custom<Result<Json<String>, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }

    let organization = match database.organization_manager.from_id(&id).await {
        Ok(Some(organization)) => organization,
        Ok(None) => {
            return Custom(
                Status::NotFound,
                Err(RequestError::from(Custom(
                    Status::NotFound,
                    "Organization not found.".into(),
                ))
                .into()),
            )
        }
        Err(err) => {
            return Custom(
                Status::InternalServerError,
                Err(RequestError::from(Custom(
                    Status::InternalServerError,
                    err.to_string(),
                ))
                .into()),
            )
        }
    };

    let project = Project {
        unique_id: Server::generate_unique_id().to_string(),
        name: project.0.name,
        organization_id: organization.unique_id.clone(),
        member_ids: Vec::new(),
    };

    match database.project_manager.create(&project).await {
        Ok(_) => (),
        Err(err) => {
            return Custom(
                Status::InternalServerError,
                Err(RequestError::from(Custom(
                    Status::InternalServerError,
                    err.to_string(),
                ))
                .into()),
            )
        }
    };

    match database
        .organization_manager
        .add_to_projects_ids(&organization.unique_id, &project.unique_id)
        .await
    {
        Ok(_) => (),
        Err(err) => {
            return Custom(
                Status::InternalServerError,
                Err(RequestError::from(Custom(
                    Status::InternalServerError,
                    err.to_string(),
                ))
                .into()),
            )
        }
    };

    Custom(Status::Ok, Ok(Json(project.unique_id)))
}

#[cfg(test)]
mod tests {
    
    use database::{group::Group, Database};
    use rocket::http::{Method, Status};
    use crate::testing::{self, dispatch_request, run_test};
    use crate::model::project_init::ProjectInit;

    // Test to create a project with a non existing organization
    #[rocket::async_test]
    async fn test_create_project_non_existing_organization() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_user = testing::get_user(database, Group::Website).await;

            let project = ProjectInit {
                name: "test_project".to_string(),
            };

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/organisations/{}/projects", "123456789"),
                Some(serde_json::to_string(&project).unwrap()),
                Some(test_user.get_token().unwrap().to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::NotFound);
        })
        .await;
    }


}