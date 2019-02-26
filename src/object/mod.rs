
mod object_id;
pub use object_id::ObjectId;

mod object_row;
pub use object_row::ObjectRow;

pub trait Object {
    fn id(&self) -> ObjectId;
}
