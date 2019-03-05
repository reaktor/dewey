use actix::prelude::*;
use futures::stream::Stream;
use rusoto_core::request::{HttpClient, TlsError};
use rusoto_core::{self, Region};
use rusoto_credential::StaticProvider;
use rusoto_s3::{self, S3Client};

/// This is object store actor
pub struct ObjectStore {
    s3: S3Client,
}

impl Actor for ObjectStore {
    type Context = Context<Self>;
}

impl ObjectStore {
    pub fn new_with_s3_credentials(
        access_key: &str,
        secret_key: &str,
    ) -> Result<ObjectStore, TlsError> {
        Ok(ObjectStore {
            s3: S3Client::new_with(
                HttpClient::new()?,
                StaticProvider::new_minimal(access_key.to_string(), secret_key.to_string()),
                Region::UsEast1,
            ),
        })
    }
}
