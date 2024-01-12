#![allow(non_snake_case)]

use crate::{layer2_fang_handler::RouteSections, layer4_ohkami::Ohkami};
use super::{Handler, IntoHandler};


pub struct Handlers {
    pub(crate) route:   RouteSections,
    pub(crate) GET:     Option<Handler>,
    pub(crate) PUT:     Option<Handler>,
    pub(crate) POST:    Option<Handler>,
    pub(crate) PATCH:   Option<Handler>,
    pub(crate) DELETE:  Option<Handler>,
} impl Handlers {
    fn new(route_str: &'static str) -> Self {
        Self {
            route:   RouteSections::from_literal(route_str),
            GET:     None,
            PUT:     None,
            POST:    None,
            PATCH:   None,
            DELETE:  None,
        }
    }
}

macro_rules! Handlers {
    ($( $method:ident ),*) => {
        impl Handlers {
            $(
                pub fn $method<Args>(mut self, handler: impl IntoHandler<Args>) -> Self {
                    self.$method.replace(handler.into_handler());
                    self
                }
            )*
        }
    };
} Handlers! { GET, PUT, POST, PATCH, DELETE }


pub struct ByAnother {
    pub(crate) route: RouteSections,
    pub(crate) ohkami: Ohkami,
}


macro_rules! Route {
    ($( $method:ident ),*) => {
        pub trait Route {
            $(
                fn $method<Args>(self, handler: impl IntoHandler<Args>) -> Handlers;
            )*
            fn By(self, another: Ohkami) -> ByAnother;
        }
        impl Route for &'static str {
            $(
                fn $method<Args>(self, handler: impl IntoHandler<Args>) -> Handlers {
                    let mut handlers = Handlers::new(self);
                    handlers.$method.replace(handler.into_handler());
                    handlers
                }
            )*
            fn By(self, another: Ohkami) -> ByAnother {
                ByAnother {
                    route:  RouteSections::from_literal(self),
                    ohkami: another,
                }
            }
        }
    };
} Route! { GET, PUT, POST, PATCH, DELETE }




#[cfg(feature="utils")]
#[cfg(test)] #[allow(unused)] mod __ {
    use std::borrow::Cow;
    use serde::{Serialize, Deserialize};
    use super::{Handlers, Route};
    use crate::{http, FromRequest, IntoResponse, Response, Request, utils};


    enum APIError {
        DBError,
    }
    impl IntoResponse for APIError {
        fn into_response(self) -> crate::Response {
            Response::InternalServerError()
        }
    }

    async fn health_check() -> http::Status {
        http::Status::NoContent
    }

    #[derive(Serialize)]
    struct User {
        id:       usize,
        name:     String,
        password: String,
    }

    mod mock {
        use super::APIError;

        pub async fn authenticate() -> Result<(), APIError> {
            Ok(())
        }

        pub const DB: __::Database = __::Database; mod __ {
            use super::APIError;

            pub struct Database;
            impl Database {
                pub async fn insert_returning_id(&self, Model: impl serde::Deserialize<'_>) -> Result<usize, APIError> {
                    Ok(42)
                }
                pub async fn update_returning_id(&self, Model: impl serde::Deserialize<'_>) -> Result<usize, APIError> {
                    Ok(24)
                }
            }
        }
    }

    #[derive(Deserialize)]
    struct CreateUser<'c> {
        name:     &'c str,
        password: &'c str,
    } impl<'req> FromRequest<'req> for CreateUser<'req> {
        type Error = crate::FromRequestError;
        fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
            let payload = req.payload().ok_or_else(|| crate::FromRequestError::Static("Payload expected"))?;
            match req.headers.ContentType() {
                Some("application/json") => serde_json::from_slice(payload).map_err(|e| crate::FromRequestError::Owned(e.to_string())),
                _ => Err(crate::FromRequestError::Static("Payload expected")),
            }
        }
    }

    async fn create_user<'req>(payload: CreateUser<'req>) -> Result<utils::JSON<User>, APIError> {
        let CreateUser { name, password } = payload;

        mock::authenticate().await?;

        let id = mock::DB.insert_returning_id(CreateUser{ name, password }).await?;

        Ok(utils::JSON::Created(User {
            id,
            name: name.to_string(),
            password: password.to_string(),
        }))
    }

    #[derive(Deserialize)]
    struct UpdateUser<'u> {
        name:     Option<&'u str>,
        password: Option<&'u str>,
    } impl<'req> FromRequest<'req> for UpdateUser<'req> {
        type Error = crate::FromRequestError;
        fn from_request(req: &'req crate::Request) -> Result<Self, Self::Error> {
            let payload = req.payload().ok_or_else(|| Self::Error::Static("Payload expected"))?;
            match req.headers.ContentType() {
                Some("application/json") => serde_json::from_slice(payload).map_err(|e| Self::Error::Owned(e.to_string())),
                _ => Err(Self::Error::Static("Payload expected")),
            }
        }
    }

    async fn update_user<'req>(body: UpdateUser<'req>) -> Result<http::Status, APIError> {
        mock::authenticate().await?;
        mock::DB.update_returning_id(body).await?;

        Ok(http::Status::NoContent)
    }


    async fn main() {
        let _ = [
            "/hc"
                .GET(health_check),
            "/api/users"
                .POST(create_user)
                .PATCH(update_user),
        ];
    }
}