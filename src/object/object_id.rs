/// Represents a ObjectId
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(DieselNewType)]
pub struct ObjectId(String);

use std::fmt;

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use crate::db::{db_error, Fetch};
use super::ObjectRow;
use actix_web::Result;
use diesel::prelude::*;
use diesel::PgConnection;

impl Fetch<ObjectRow> for ObjectId {
    fn fetch(&self, conn: &PgConnection) -> Result<ObjectRow> {
        use crate::db::schema::objects::dsl::*;

        objects
            .filter(id.eq(&self.0))
            .get_result::<ObjectRow>(conn)
            .map_err(|e| db_error("db select get object by id error", e))
    }
}
