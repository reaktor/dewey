use ::actix::prelude::*;
use ::actix_web::{error, FutureResponse, Result};

use futures::future::{self, Future};

use rusoto_core::request::{HttpClient, TlsError};
use rusoto_core::{self, Region};
use rusoto_credential::{ProvideAwsCredentials, StaticProvider};
use rusoto_s3::util::{PreSignedRequest, PreSignedRequestOption};
use rusoto_s3::{self, S3Client};

/// This is object store actor
pub struct ObjectStore {
    s3: S3Client,
    creds: StaticProvider,
    region: Region,
}

impl Actor for ObjectStore {
    type Context = Context<Self>;
}

impl ObjectStore {
    pub fn new_with_s3_credentials(
        access_key: &str,
        secret_key: &str,
    ) -> Result<ObjectStore, TlsError> {
        let creds = StaticProvider::new_minimal(access_key.to_string(), secret_key.to_string());
        let region = Region::Custom {
            name: "murica-east-1".to_string(),
            endpoint: dotenv!("S3_URL").to_string(),
        };
        Ok(ObjectStore {
            creds: creds.clone(),
            region: region.clone(),
            s3: S3Client::new_with(HttpClient::new()?, creds, region),
        })
    }
}

pub struct GetPutUrl {
    pub bucket: String,
}

pub struct PutUrl {
    pub url: String,
    pub bucket: String,
    pub key: String,
}

impl Message for GetPutUrl {
    type Result = Result<PutUrl>;
}

impl Handler<GetPutUrl> for ObjectStore {
    type Result = ResponseFuture<PutUrl, actix_web::Error>;

    fn handle(&mut self, msg: GetPutUrl, _: &mut Self::Context) -> Self::Result {
        let region = self.region.clone();
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

                    put_object_req.bucket = msg.bucket;
                    // TODO: Fix this for collisions and sort based on content hash
                    // TODO: 1. Upload, 2. Hash 3. Merge with existing
                    put_object_req.key = crate::sessions::rand_util::random_string(12);

                    PutUrl {
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
