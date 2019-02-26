/// Represents a SelectChoiceId
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(DieselNewType)]
pub struct SelectChoiceId(i64);

use std::fmt;

impl fmt::Display for SelectChoiceId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
