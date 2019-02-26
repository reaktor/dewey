/// Represents a PropertyId
#[derive(DieselNewType)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyId(i64);

use std::fmt;

impl fmt::Display for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
