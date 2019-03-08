use ::chrono::{DateTime, Utc};

use super::{SelectChoice, SelectChoiceId};
use crate::user::UserId;
use crate::property::PropertyId;

/// table "property_value_choices"
#[derive(Debug, Queryable)]
pub struct SelectChoiceRow {
    pub id: SelectChoiceId,
    pub property_id: PropertyId,
    pub display: String,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
}

impl SelectChoice for SelectChoiceRow {
    fn id(&self) -> SelectChoiceId {
        self.id.clone()
    }
    fn property_id(&self) -> PropertyId {
        self.property_id.clone()
    }
}