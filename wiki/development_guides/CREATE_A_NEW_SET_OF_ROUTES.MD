[return to list of guides](../README.MD)
# Create a new set of routes
> If you want to implement a new set of route on the rest api, you can do it by following these steps:

**1. Create a new file that will contain your routes `server/crates/api/src/route/set_of_route_name.rs`**

**2. Then add a test route to test if it works**
```rust
use rocket::*;
use rocket_okapi::openapi;

/// Test route
#[openapi(tag = "SetOfRouteName")] // the tag is here to regroup all the routes that have the same tag name.
#[get("/route_address")] // <- route attribute
pub async fn test() -> String {
    "Service name test!".to_string()
}
```

**3. Register your route at: `server/crates/api/src/route/mod.rs`**
```rust
impl ApiRoute {
    pub fn retrieve_routes(&self) -> (Vec<Route>, OpenApi) {
        match self {
            Self::User => openapi_get_routes_spec![
                // ...,
                test,
                // ...,
            ],
            // ...,
        }
    }
}
```


```rs
You're done, you are now able to make a request at /route_address
```

[return to list of guides](../README.MD)