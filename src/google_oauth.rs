use chrono::{DateTime, Utc, Duration};

#[derive(Clone, Debug)]
pub struct GoogleAccessToken {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}
