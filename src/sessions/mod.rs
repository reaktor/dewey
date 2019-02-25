use super::db;

mod oauth;
mod rand_util;

pub mod session_manager;
pub mod session_routes;

pub use oauth::GoogleAccessToken;

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSessionKey {
    #[serde(rename = "i")]
    user_id: i64,
    #[serde(rename = "v")]
    version: i32,
}

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserSession {
    #[serde(rename = "k")]
    key: UserSessionKey,
    #[serde(rename = "e")]
    email_address: String,
    #[serde(rename = "d")]
    display_name: String,
}
