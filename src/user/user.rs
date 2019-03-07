use super::{UserId, UserKind, UserRow};

/// The common set of methods for a user
pub trait User {
    fn id(&self) -> UserId;
    fn kind(&self) -> &UserKind;
    fn display_name(&self) -> &str;
    fn full_name(&self) -> &str;
    fn photo_url(&self) -> Option<&String>;
    fn public_email(&self) -> Option<&String>;
}

use crate::db::Fetch;
use actix_web::Result;
use diesel::PgConnection;

/// Everything that implements User also implements Fetch<UserRow>
impl<U: User> Fetch<UserRow> for U {
    fn fetch(&self, conn: &PgConnection) -> Result<UserRow> {
        self.id().fetch(&conn)
    }
}
