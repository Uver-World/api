use database::{authentication::Authentication, authentication::Credentials, group::Group, managers::UserManager, Database};
use rocket::{http::Status, post, response::status::Custom, serde::json::Json, State};
use rocket_okapi::openapi;
use gravatar::{Gravatar, Rating};

use crate::{
    model::{api_socket_addr::ApiSocketAddr, login::Login, user_token::UserData},
    RequestError, Server,
};

/// Register a new user
///
/// If "Credentials" is being used, the user will have an authentication method of type credentials by default
///
/// Otherwise please don't use any
///
/// Requires 'Website' group
#[openapi(tag = "Users")]
#[post("/", data = "<login>", format = "application/json")] // <- route attribute
pub async fn register(
    user_data: UserData,
    database: &State<Database>,
    login: Option<Json<Login>>,
    remot_addr: ApiSocketAddr,
) -> Custom<Result<String, Json<RequestError>>> {
    if let Err(response) = user_data.matches_group(vec![Group::Website]) {
        return Custom(response.0, Err(RequestError::from(response).into()));
    }
    let ip = remot_addr.0.ip().to_string();
    if login.is_none() {
        return _register(Authentication::None, ip, &database.user_manager).await;
    }
    
    let login = login.unwrap();
    
    match login.0 {
        Login::Credentials(credentials) => {
            let url = Gravatar::new(credentials.email.as_str())
                .set_size(Some(150))
                .set_rating(Some(Rating::Pg))
                .set_default(Some(gravatar::Default::Retro))
                .image_url();

            // Get the image url and store it in the credentials.avatar
            // let avatar = url.to_string();
            let credentials = Credentials {
                email: credentials.email,
                username: credentials.username,
                avatar: Some(url.to_string()),
                password: credentials.password,
            };

            let auth = Authentication::Credentials(credentials.clone());

            _register(auth, ip, &database.user_manager).await
        }
        _ => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(
                Status::BadRequest,
                "Credentials are required.".to_string(),
            ))
            .into()),
        ),
    }
}

async fn _register(
    auth: Authentication,
    ip: String,
    usermanager: &UserManager,
) -> Custom<Result<String, Json<RequestError>>> {
    let result = auth
        .register(
            Server::current_time(),
            Server::generate_unique_id().to_string(),
            &usermanager.users,
        )
        .await;
    match result {
        Ok(user) if user.is_some() => {
            let user = user.unwrap();
            let login = database::login::Login::new(ip, Server::current_time(), auth);

            user.upload_token(&login, &usermanager.users).await;

            Custom(Status::Ok, Ok(login.token.0))
        }
        Ok(_) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(Status::Conflict, "User already exists.".into())).into()),
        ),
        Err(error) => Custom(
            Status::Ok,
            Err(RequestError::from(Custom(Status::InternalServerError, error)).into()),
        ),
    }
}

#[cfg(test)]
mod tests {

    use database::{
        authentication::{Authentication, Credentials},
        group::Group,
        Database,
    };
    use rocket::http::{Method, Status};

    use crate::{
        model::login::Login,
        testing::{self, dispatch_request, run_test},
    };

    #[rocket::async_test]
    async fn test_register() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();
            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: Option::Some("test".to_string()),
                avatar: Option::Some("test".to_string()),
                password: "test".to_string(),
            };

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user"),
                Some(serde_json::to_string(&Login::Credentials(credentials.clone())).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let user_token = response.into_string().await.unwrap();
            let user = database
                .user_manager
                .from_token(&user_token)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                user.authentication,
                Authentication::Credentials(credentials)
            );
        })
        .await;
    }

    #[rocket::async_test]
    async fn test_no_auth_register() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, Group::Website).await;
            let request_token = request_user.get_token().unwrap();

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user"),
                None,
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Ok);
            let user_token = response.into_string().await.unwrap();
            let user = database
                .user_manager
                .from_token(&user_token)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(user.authentication, Authentication::None);
        })
        .await;
    }

    #[rocket::async_test]
    async fn unauthorized_test_register() {
        _unauthorized_test_register(Group::User).await;
        _unauthorized_test_register(Group::Server).await;
    }

    async fn _unauthorized_test_register(request_group: Group) {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();
            let request_user = testing::get_user(database, request_group).await;
            let request_token = request_user.get_token().unwrap();

            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: Option::Some("test".to_string()),
                avatar: Option::Some("test".to_string()),
                password: "test".to_string(),
            };

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user"),
                Some(serde_json::to_string(&Login::Credentials(credentials.clone())).unwrap()),
                Some(request_token.to_string()),
            )
            .await;

            assert_eq!(response.status(), Status::Unauthorized);
            // Email should not exist because the request was unauthorized.
            assert!(!database
                .user_manager
                .email_exists(credentials.email)
                .await
                .unwrap());
        })
        .await;
    }

    #[rocket::async_test]
    async fn forbidden_test_register() {
        run_test(|client| async move {
            let database = client.rocket().state::<Database>().unwrap();

            let credentials = Credentials {
                email: "test@test.fr".to_string(),
                username: Option::Some("test".to_string()),
                avatar: Option::Some("test".to_string()),
                password: "test".to_string(),
            };

            let response = dispatch_request(
                &client,
                Method::Post,
                format!("/user"),
                Some(serde_json::to_string(&Login::Credentials(credentials.clone())).unwrap()),
                None,
            )
            .await;

            assert_eq!(response.status(), Status::Forbidden);
            // Email should not exist because the request was forbidden.
            assert!(!database
                .user_manager
                .email_exists(credentials.email)
                .await
                .unwrap());
        })
        .await;
    }
}
