use ::actix::prelude::*;
use ::actix_web::{error, FutureResponse, Result};

use futures::future::{self, Future};
use futures::stream::Stream;

use rusoto_core::request::{HttpClient, TlsError};
use rusoto_core::{self, Region};
use rusoto_credential::{ProvideAwsCredentials, StaticProvider};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::{self, S3Client, S3};

use std::sync::Arc;

/// This is object store actor
pub struct ObjectStore {
    s3: Arc<S3Client>,
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
            s3: Arc::new(s3),
            pending_bucket: pending_bucket,
            collect_bucket: collect_bucket,
        })
    }

    fn get_credentials(
        &self,
    ) -> ResponseFuture<rusoto_credential::AwsCredentials, actix_web::Error> {
        Box::new(
            self.creds
                .credentials()
                .map_err(|_| error::ErrorInternalServerError("Error retrieving S3 credentials")),
        )
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
        Box::new(self.get_credentials().map(
            move |credentials: rusoto_credential::AwsCredentials| {
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
            },
        ))
    }
}

/// When you just gotta get that file into the cloud,
/// then figure out where it goes later.
pub struct FinalizeObject {
    pub key: String,
}

pub struct StoredObject {
    pub key: String,
    pub bucket: String,
}

impl Message for FinalizeObject {
    type Result = Result<StoredObject>;
}

impl Handler<FinalizeObject> for ObjectStore {
    type Result = ResponseFuture<StoredObject, actix_web::Error>;

    fn handle(&mut self, msg: FinalizeObject, _: &mut Self::Context) -> Self::Result {
        let region = self.region.clone();
        let collect_bucket = self.collect_bucket.clone();
        let pending_bucket = self.pending_bucket.clone();
        let pending_bucket_1 = self.pending_bucket.clone();
        let s3_client = self.s3.clone();
        let s3_client_1 = self.s3.clone();
        let key = msg.key.clone();

        Box::new(
            {
                // first hash the object file
                let pending_bucket = pending_bucket_1;
                let mut get_object_request = rusoto_s3::GetObjectRequest::default();
                get_object_request.key = key;
                get_object_request.bucket = pending_bucket;

                s3_client
                    .get_object(get_object_request)
                    .map_err(|e| error::ErrorNotFound(e))
                    .and_then(|obj: rusoto_s3::GetObjectOutput| {
                        use ring::digest::{Context, SHA256};
                        match obj.body {
                            Some(stream) => {
                                let context = Context::new(&SHA256);
                                future::Either::A(
                                    stream
                                        .fold(context, |mut ctx: Context, bytes: Vec<u8>| {
                                            ctx.update(&bytes[..]);
                                            future::ok::<_, std::io::Error>(ctx)
                                        })
                                        .map(|ctx| {
                                            let dig = ctx.finish();
                                            // create the key
                                            format!("sha256-{}", hex(dig.as_ref()))
                                        })
                                        .map_err(|_| {
                                            error::ErrorInternalServerError(
                                                "Error hashing object from s3",
                                            )
                                        }),
                                )
                            }
                            None => future::Either::B(future::err(error::ErrorNotFound(
                                "Body not found for s3 object",
                            ))),
                        }
                    })
            }
            .and_then(move |key: String| {
                // move the object file
                let mut copy_object_req = rusoto_s3::CopyObjectRequest::default();

                copy_object_req.bucket = collect_bucket.clone();
                copy_object_req.key = key.clone();
                copy_object_req.copy_source = format!("{}/{}", pending_bucket, msg.key);

                s3_client_1
                    .copy_object(copy_object_req)
                    .map_err(|e| {
                        error::ErrorInternalServerError(format!("Failed to copy s3 object {:?}", e))
                    })
                    .map(move |out| {
                        info!("Copied object! {:?}", out);
                        StoredObject {
                            bucket: collect_bucket,
                            key: key,
                        }
                    })
            }),
        )
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    for &byte in bytes {
        write!(&mut s, "{:x}", byte).expect("Unable to write");
    }
    s
}
