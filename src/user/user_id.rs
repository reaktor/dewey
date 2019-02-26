#[derive(Debug, Clone, Serialize, Deserialize, DieselNewType)]
pub struct UserId(i64);

use std::fmt;

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use crate::db::{db_error, Fetch};
use crate::user::UserRow;
use actix_web::Result;
use diesel::prelude::*;
use diesel::PgConnection;

impl Fetch<UserRow> for UserId {
    fn fetch(&self, conn: &PgConnection) -> Result<UserRow> {
        use crate::db::schema::users::dsl::*;

        users
            .filter(id.eq(&self.0))
            .get_result::<UserRow>(conn)
            .map_err(|e| db_error("db select get user by id error", e))
    }
}
