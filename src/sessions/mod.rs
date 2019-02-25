use super::db;

mod oauth;
mod rand_util;

pub mod session_manager;
pub mod session_routes;

pub use oauth::GoogleAccessToken;

pub use db::{PersonUser, UserSessionKey};

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    #[serde(rename = "k")]
    pub key: UserSessionKey,
    #[serde(rename = "p")]
    pub person: PersonUser,
}