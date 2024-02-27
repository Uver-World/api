mod organization;
mod user;

use rocket::Route;
use rocket_okapi::okapi::openapi3::OpenApi;
use rocket_okapi::openapi_get_routes_spec;

use crate::cors;

pub enum ApiRoute {
    Root,
    User,
    Organization,
}

impl ApiRoute {
    pub fn retrieve_routes(&self) -> (Vec<Route>, OpenApi) {
        match self {
            Self::Root => openapi_get_routes_spec![cors::cors_options],
            Self::User => openapi_get_routes_spec![
                user::create_license,
                user::get_organizations,
                user::renew,
                user::register,
                user::email_exists,
                user::from_id,
                user::from_token,
                user::update_auth,
                user::update,
                user::delete_from_id,
                user::delete_from_token,
                user::server_authenticate,
                user::access_server,
                user::has_access,
                user::server_disconnect,
                user::from_email,
            ],
            Self::Organization => openapi_get_routes_spec![
                organization::add_member,
                organization::from_id,
                organization::delete_from_id,
                organization::create,
                organization::create_project,
                organization::update,
                organization::add_server,
                organization::remove_server,
                organization::remove_member,
                organization::get_projects_from_organization,
                organization::delete_project,
                organization::project_from_id,
                organization::update_project,
            ],
        }
    }
}
