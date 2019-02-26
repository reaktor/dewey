// Schema definition

// define your enum
/// property_type enum
#[derive(Debug, DbEnum)]
pub enum PropertyType {
    Timestamptz,  // All variants must be fieldless
    Text,
    Relation,
    Choice,
}
