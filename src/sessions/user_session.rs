pub use crate::db::UserSessionKey;
pub use crate::user::PersonUser;

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    #[serde(rename = "k")]
    pub key: UserSessionKey,
    #[serde(rename = "p")]
    pub person: PersonUser,
}
