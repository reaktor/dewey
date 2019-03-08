use ::chrono::{DateTime, Utc};

use crate::user::UserId;
use crate::property::PropertyId;
use crate::object::ObjectId;

use super::RelationValue;

/// table "relation_values"
#[derive(Debug, Queryable)]
pub struct RelationValueRow {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub target_id: ObjectId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl RelationValue for RelationValueRow {
    fn object_id(&self) -> ObjectId {
        self.object_id.clone()
    }
    fn property_id(&self) -> PropertyId {
        self.property_id.clone()
    }
    fn target_id(&self) -> ObjectId {
        self.target_id.clone()
    }
}
