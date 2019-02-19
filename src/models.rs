
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
    pub created_by: i64,
    pub created_at: std::time::SystemTime,
}

#[derive(Queryable)]
pub struct Property {
    pub id: i64,
    pub created_by: i64,
    pub created_at: std::time::SystemTime,
    pub display: String,
    pub type_: String,
}

#[derive(Queryable)]
pub struct PropertySelectChoice {
    pub id: i32,
    pub property_id: i64,
    pub display: String,
    pub created_by: i64,
    pub created_at: std::time::SystemTime,
}

#[derive(Queryable)]
pub struct Values {
    pub object_id: String,
    pub property_id: i64,
    pub created_by: i64,
    pub created_at: std::time::SystemTime,
    pub value: String,
}
