mod user_id;
pub use user_id::UserId;

mod user_row;
pub use user_row::UserRow;

mod person_user;
pub use person_user::PersonUser;

mod user;
pub use user::User;

/// user_kind enum
/// This maps directly to a PostgreSQL enum type
#[derive(Debug, Clone, DbEnum)]
pub enum UserKind {
    Person,
    Reserved,
    Plugin,
}
