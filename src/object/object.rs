use super::ObjectId;

pub trait Object {
    fn id(&self) -> ObjectId;
}
