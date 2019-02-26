use super::{User, UserId, UserKind};

#[derive(Debug, Queryable)]
pub struct UserRow {
    id: UserId,
    google_resource_id: Option<String>,
    full_name: String,
    display_name: String,
    public_email: Option<String>,
    kind: UserKind,
    photo_url: Option<String>,
}

impl User for UserRow {
    fn id(&self) -> UserId {
        self.id.clone()
    }
    fn kind(&self) -> &UserKind {
        &self.kind
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
        self.public_email.as_ref()
    }
}
