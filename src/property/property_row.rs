use ::chrono::{DateTime, Utc};

use super::{Property, PropertyId, PropertyType};
use crate::user::UserId;

#[derive(Debug, Queryable)]
pub struct PropertyRow {
    pub id: PropertyId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub ord: f32,
    pub display: String,
    pub property_type: PropertyType,
}
