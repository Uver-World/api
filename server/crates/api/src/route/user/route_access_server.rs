use database::{peer::Peer, Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;

use crate::{
    model::{organisation_id::OrganizationId, user_token::UserData},
    RequestError,
};

#[openapi(tag = "Users")]
#[post("/access_server", data = "<organisation_id>", format = "application/json")] // <- route attribute
pub async fn access_server(
    user_data: UserData,
    database: &State<Database>,
    organisation_id: Json<OrganizationId>,
) -> Custom<Result<Json<Peer>, Json<RequestError>>> {


    let organisation_id = organisation_id.0.0;

    let is_in_org = database
        .organization_manager
        .is_in_organization(&organisation_id, &user_data.id.unwrap())
        .await
        .unwrap();

    if !is_in_org {
        return Custom(
            Status::Forbidden,
            Err(RequestError::from(Custom(
                Status::Forbidden,
                format!("User is not in the organisation."),
            ))
            .into()),
        );
    }

    let servers_ids = database
        .organization_manager
        .get_servers_ids_from_organisation(&organisation_id)
        .await
        .unwrap();
    
    for server_id in servers_ids {
        match database.peers_manager.from_server_id(&server_id).await {
            Ok(Some(peer)) => return Custom(Status::Ok, Ok(Json(peer))),
            _ => continue,
        }
    }

    Custom(
        Status::Ok,
        Err(RequestError::from(Custom(
            Status::NotFound,
            format!("No server is currently online."),
        ))
        .into()),
    )
}

#[cfg(test)]
mod tests {

    use database::{peer::Peer, Database};
    use rocket::http::{Method, Status};

    use crate::{
        model::server_id::ServerId,
        testing::{self, dispatch_request, run_test},
        Server,
    };

    #[rocket::async_test]
    async fn test_access_server() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let test_server = testing::get_user(database).await;
            let test_user = testing::get_user(database).await;
            let _test_org =
                testing::create_org(database, &test_user, vec![test_server.unique_id.clone()])
                    .await;

            let server_peer = Peer {
                room_id: Server::generate_unique_id().to_string(),
                creation_date: Server::current_time().to_string(),
                signaling_hostname: "127.0.0.1".to_string(),
                signaling_port: 3536,
                server_unique_id: test_server.unique_id.clone(),
            };

            let _ = database
                .peers_manager
                .create_peer(&server_peer)
                .await
                .unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user/access_server"),
                Some(serde_json::to_string(&ServerId(test_server.unique_id)).unwrap()),
                Some(test_user.get_token().unwrap().to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let _ = response.into_json::<Peer>().await.unwrap();
        })
        .await;
    }
}
