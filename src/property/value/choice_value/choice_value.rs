use crate::object::ObjectId;
use crate::property::{PropertyId, SelectChoiceId};

pub trait ChoiceValue {
    fn object_id(&self) -> ObjectId;
    fn property_id(&self) -> PropertyId;
    fn value_id(&self) -> SelectChoiceId;
    fn display(&self) -> String;
}
