use ::chrono::{DateTime, Utc};

use super::{Object, ObjectId};
use crate::user::UserId;

#[derive(Debug, Queryable)]
pub struct ObjectRow {
    pub id: ObjectId,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub extension: String,
    pub level: i32,
}

impl Object for ObjectRow {
    fn id(&self) -> ObjectId {
        self.id.clone()
    }
}
