mod property_type;
pub use property_type::{PropertyType, PropertyTypeMapping};

mod select_choice;
pub use select_choice::SelectChoiceId;

mod property_id;
pub use property_id::PropertyId;

pub mod value;

use crate::user::UserId;

pub trait Property {
    fn id(&self) -> PropertyId;
    fn display(&self) -> &str;
    fn kind(&self) -> &PropertyType;
    fn created_by(&self) -> &UserId;
}
