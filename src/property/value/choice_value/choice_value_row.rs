use ::chrono::{DateTime, Utc};

use crate::user::UserId;
use crate::property::{PropertyId, SelectChoiceId};
use crate::object::ObjectId;

use super::ChoiceValue;

/// table "choice_values"
#[derive(Debug, Queryable)]
pub struct ChoiceValueRow {
    pub object_id: ObjectId,
    pub property_id: PropertyId,
    pub value_id: SelectChoiceId,
    pub display: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl ChoiceValue for ChoiceValueRow {
    fn object_id(&self) -> ObjectId {
        self.object_id.clone()
    }
    fn property_id(&self) -> PropertyId {
        self.property_id.clone()
    }
    fn value_id(&self) -> SelectChoiceId {
        self.value_id.clone()
    }
    fn display(&self) -> String {
        self.display.clone()
    }
}
