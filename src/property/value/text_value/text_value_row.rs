use ::chrono::{DateTime, Utc};

use crate::user::UserId;
use crate::property::PropertyId;
use crate::object::ObjectId;

use super::TextValue;

/// table "text_values"
#[derive(Debug, Queryable)]
pub struct TextValueRow {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub value: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl TextValue for TextValueRow {
    fn object_id(&self) -> ObjectId {
        self.object_id.clone()
    }
    fn property_id(&self) -> PropertyId {
        self.property_id.clone()
    }
    fn value(&self) -> String {
        self.value.clone()
    }
}
