use super::SelectChoiceId;
use crate::property::PropertyId;

pub trait SelectChoice {
    fn id(&self) -> SelectChoiceId;
    fn property_id(&self) -> PropertyId;
}
