use super::{User, UserId, UserKind};
use actix_web::{error, Result};

/// Valid User Session comprises of the session's user_id and the user's version
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonUser {
    #[serde(rename = "i")]
    pub user_id: UserId,
    #[serde(rename = "pe")]
    pub public_email: String,
    #[serde(rename = "dn")]
    pub display_name: String,
    #[serde(rename = "fn")]
    pub full_name: String,
    #[serde(rename = "pu")]
    pub photo_url: Option<String>,
}

impl PersonUser {
    /// Try from will fail if the user does not have a public email
    pub fn try_from<U: User>(user: &U) -> Result<Self> {
        Ok(PersonUser {
            user_id: user.id(),
            display_name: user.display_name().to_string(),
            full_name: user.full_name().to_string(),
            public_email: user
                .public_email()
                .ok_or_else(|| {
                    error::ErrorExpectationFailed("db person should have public_email error")
                })?
                .to_owned(),
            photo_url: user.photo_url().cloned(),
        })
    }
}

const PERSON_KIND: &UserKind = &UserKind::Person;

impl User for PersonUser {
    fn id(&self) -> UserId {
        self.user_id.clone()
    }
    fn kind(&self) -> &UserKind {
        &PERSON_KIND
    }
    fn display_name(&self) -> &str {
        &self.display_name
    }
    fn full_name(&self) -> &str {
        &self.full_name
    }
    fn photo_url(&self) -> Option<&String> {
        self.photo_url.as_ref()
    }
    fn public_email(&self) -> Option<&String> {
        Some(&self.public_email)
    }
}
