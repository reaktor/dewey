use ::actix::prelude::*;
use ::actix_web::{error, FutureResponse, Result};

use futures::future::{self, Future};

use rusoto_core::request::{HttpClient, TlsError};
use rusoto_core::{self, Region};
use rusoto_credential::{ProvideAwsCredentials, StaticProvider};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::{self, S3Client, S3};

/// This is object store actor
pub struct ObjectStore {
    s3: S3Client,
    creds: StaticProvider,
    region: Region,
    pending_bucket: String,
    collect_bucket: String,
}

impl Actor for ObjectStore {
    type Context = Context<Self>;
}

#[derive(Debug)]
pub enum ObjectStoreCreationError {
    TlsError(TlsError),
    CreateBucketError(rusoto_s3::CreateBucketError),
}

impl ObjectStore {
    pub fn new_with_s3_credentials(
        access_key: &str,
        secret_key: &str,
        bucket_prefix: &str,
    ) -> Result<ObjectStore, ObjectStoreCreationError> {
        let creds = StaticProvider::new_minimal(access_key.to_string(), secret_key.to_string());
        let region_name = "murica-east-1";
        let region = Region::Custom {
            name: region_name.to_string(),
            endpoint: dotenv!("S3_URL").to_string(),
        };

        let pending_bucket = format!("{}-pending", bucket_prefix);
        let collect_bucket = format!("{}-collect", bucket_prefix);

        let http_client =
            HttpClient::new().map_err(|tls_err| ObjectStoreCreationError::TlsError(tls_err))?;
        let s3 = S3Client::new_with(http_client, creds.clone(), region.clone());

        let mut create_pending_bucket = rusoto_s3::CreateBucketRequest::default();
        create_pending_bucket.bucket = pending_bucket.clone();

        let mut create_collect_bucket = rusoto_s3::CreateBucketRequest::default();
        create_collect_bucket.bucket = collect_bucket.clone();

        match s3.create_bucket(create_pending_bucket).sync() {
            Err(rusoto_s3::CreateBucketError::BucketAlreadyOwnedByYou(_)) => (),
            res => {
                res.map_err(|create_bucket_err| {
                    ObjectStoreCreationError::CreateBucketError(create_bucket_err)
                })?;
            }
        }
        match s3.create_bucket(create_collect_bucket).sync() {
            Err(rusoto_s3::CreateBucketError::BucketAlreadyOwnedByYou(_)) => (),
            res => {
                res.map_err(|create_bucket_err| {
                    ObjectStoreCreationError::CreateBucketError(create_bucket_err)
                })?;
            }
        }

        Ok(ObjectStore {
            creds: creds,
            region: region,
            s3: s3,
            pending_bucket: pending_bucket,
            collect_bucket: collect_bucket,
        })
    }
}

/// When you just gotta get that file into the cloud,
/// then figure out where it goes later.
pub struct GetPendingPutUrl {
    pub user_id: crate::user::UserId,
}

pub struct PendingPutUrl {
    pub url: String,
    pub bucket: String,
    pub key: String,
}

impl Message for GetPendingPutUrl {
    type Result = Result<PendingPutUrl>;
}

impl Handler<GetPendingPutUrl> for ObjectStore {
    type Result = ResponseFuture<PendingPutUrl, actix_web::Error>;

    fn handle(&mut self, msg: GetPendingPutUrl, _: &mut Self::Context) -> Self::Result {
        let region = self.region.clone();
        let pending_bucket = self.pending_bucket.clone();
        Box::new(
            self.creds
                .credentials()
                .map_err(|_| error::ErrorInternalServerError("Error retrieving S3 credentials"))
                .map(move |credentials: rusoto_credential::AwsCredentials| {
                    let mut put_object_req = rusoto_s3::PutObjectRequest::default();
                    let pre_signed_req_opts = PreSignedRequestOption {
                        // 20 second expiration
                        expires_in: std::time::Duration::from_secs(20),
                    };

                    put_object_req.bucket = pending_bucket;
                    // TODO: Fix this for collisions and sort based on content hash
                    // TODO: 1. Upload, 2. Hash 3. Merge with existing
                    put_object_req.key = format!(
                        "{}-{}-{}",
                        chrono::Utc::now().format("%F"),
                        msg.user_id,
                        crate::sessions::rand_util::random_string(6)
                    );

                    PendingPutUrl {
                        url: put_object_req.get_presigned_url(
                            &region,
                            &credentials,
                            &pre_signed_req_opts,
                        ),
                        bucket: put_object_req.bucket,
                        key: put_object_req.key,
                    }
                }),
        )
    }
}
