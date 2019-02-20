
#[derive(Queryable)]
pub struct User {
    pub id: i64,
    pub google_resource_id: Option<String>,
    pub full_name: String,
    pub display_name: String,
}

#[derive(Queryable)]
pub struct Object {
    pub id: String,
    pub extension: String,
    pub created_by: i64,
    pub created_at: std::time::SystemTime,
}
