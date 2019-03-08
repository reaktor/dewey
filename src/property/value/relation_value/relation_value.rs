use ::chrono::{DateTime, Utc};

use crate::object::ObjectId;
use crate::property::PropertyId;

pub trait RelationValue {
    fn object_id(&self) -> ObjectId;
    fn property_id(&self) -> PropertyId;
    fn target_id(&self) -> ObjectId;
}
