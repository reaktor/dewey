use ::chrono::{DateTime, Utc};

use crate::object::ObjectId;
use crate::property::PropertyId;

pub trait TimestamptzValue {
    fn object_id(&self) -> ObjectId;
    fn property_id(&self) -> PropertyId;
    fn value(&self) -> Option<&DateTime<Utc>>;
}
