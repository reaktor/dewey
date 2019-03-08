use ::chrono::{DateTime, Utc};

use crate::user::UserId;
use crate::property::PropertyId;
use crate::object::ObjectId;

use super::TimestamptzValue;

/// table "timestamptz_values"
#[derive(Debug, Queryable)]
pub struct TimestamptzValueRow {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub value: Option<DateTime<Utc>>,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl TimestamptzValue for TimestamptzValueRow {
    fn object_id(&self) -> ObjectId {
        self.object_id.clone()
    }
    fn property_id(&self) -> PropertyId {
        self.property_id.clone()
    }
    fn value(&self) -> Option<&DateTime<Utc>> {
        self.value.as_ref()
    }
}
