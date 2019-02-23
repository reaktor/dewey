table! {
    view_user_id_token_versions (google_resource_id) {
        user_id -> Int8,
        google_resource_id -> Text,
        version -> Int4,
    }
}

#[derive(Queryable)]
pub struct ViewUserIdTokenVersion {
    pub user_id: i64,
    pub google_resource_id: String,
    pub version: i32,
}

use diesel::prelude::*;
use diesel::PgConnection;

pub fn get_user_token_versions_by_resource(
    conn: &PgConnection,
    resource_id: &str,
) -> Result<Option<ViewUserIdTokenVersion>, diesel::result::Error> {
    use self::view_user_id_token_versions::dsl::*;

    view_user_id_token_versions
        .filter(google_resource_id.eq(resource_id))
        .get_result(conn)
        .optional()
}
