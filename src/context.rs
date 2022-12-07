#[cfg(feature = "sqlx")]
use async_std::sync::Arc;

use serde::Deserialize;
use crate::{
    components::json::JSON,
    response::Response, result::Result,
};

#[cfg(feature = "postgres")]
use sqlx::PgPool as ConnectionPool;
#[cfg(feature = "mysql")]
use sqlx::MySqlPool as ConnectionPool;

#[cfg(not(feature = "sqlx"))]
#[derive(Debug)]
pub struct Context {
    pub       param: Option<u32>,  // Option<&'ctx str>,
    pub(crate) body: Option<JSON>,
}
#[cfg(not(feature = "sqlx"))]
impl<'d> Context {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
}

#[cfg(feature = "sqlx")]
#[derive(Debug)]
pub struct Context {
    pub(crate) pool: Arc<ConnectionPool>,
    pub       param: Option<u32>,  // Option<&'ctx str>,
    pub(crate) body: Option<JSON>,
}
#[cfg(feature = "sqlx")]
impl<'d> Context {
    pub fn request_body<D: Deserialize<'d>>(&'d self) -> Result<D> {
        let json = self.body.as_ref()
            .ok_or_else(|| Response::BadRequest("expected request body"))?;
        let json_struct = json.to_struct()?;
        Ok(json_struct)
    }
    pub fn pool(&self) -> &ConnectionPool {
        &*self.pool
    }
}
