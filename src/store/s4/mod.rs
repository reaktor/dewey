//! Simpler Simple Storage Service: high-level API extensions for Rusoto's `S3Client`
//!
//! # TLS Support
//!
//! For TLS support [`rustls`](https://crates.io/crates/rustls) is used by default.
//!
//! Alternatively, [`native-tls`](https://crates.io/crates/native-tls) can be used by
//! disabling the default features and enabling the `native-tls` feature.
//!
//! For instance, like this:
//!
//! ```toml
//! [dependencies]
//! s4 = { version = "…", default_features = false }
//!
//! [features]
//! default = ["s4/native-tls"]
//! ```

#![allow(clippy::default_trait_access)]
#![allow(clippy::stutter)]

extern crate fallible_iterator;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_s3;

pub mod iter;
use self::iter::{GetObjectIter, ObjectIter};
pub mod error;
use self::error::{S4Error, S4Result};
mod upload;

use futures::stream::Stream;
use rusoto_core::request::{HttpClient, TlsError};
use rusoto_core::Region;
use rusoto_credential::StaticProvider;
use rusoto_s3::{
    CompleteMultipartUploadOutput, GetObjectOutput, GetObjectRequest, PutObjectOutput,
    PutObjectRequest, S3Client, StreamingBody, S3,
};
use std::convert::AsRef;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

/// Create client using given static access/secret keys
pub fn new_s3client_with_credentials(
    region: Region,
    access_key: String,
    secret_key: String,
) -> Result<S3Client, TlsError> {
    Ok(S3Client::new_with(
        HttpClient::new()?,
        StaticProvider::new_minimal(access_key, secret_key),
        region,
    ))
}

pub trait S4 {
    /// Get object and write it to file `target`
    fn download_to_file<F>(&self, source: GetObjectRequest, target: F) -> S4Result<GetObjectOutput>
    where
        F: AsRef<Path>;

    /// Upload content of file to S3
    ///
    /// # Caveats
    ///
    /// The current implementation is incomplete. For now, the following limitation applies:
    ///
    /// * The full content of `source` is copied into memory.
    fn upload_from_file<F>(&self, source: F, target: PutObjectRequest) -> S4Result<PutObjectOutput>
    where
        F: AsRef<Path>;

    /// Upload content of file to S3 using multi-part upload
    ///
    /// # Caveats
    ///
    /// The current implementation is incomplete. For now, the following limitation applies:
    ///
    /// * The full content of a part is copied into memory.
    fn upload_from_file_multipart<F>(
        &self,
        source: F,
        target: &PutObjectRequest,
        part_size: usize,
    ) -> S4Result<CompleteMultipartUploadOutput>
    where
        F: AsRef<Path>;

    /// Get object and write it to `target`
    fn download<W>(&self, source: GetObjectRequest, target: &mut W) -> S4Result<GetObjectOutput>
    where
        W: Write;

    /// Read `source` and upload it to S3
    ///
    /// # Caveats
    ///
    /// The current implementation is incomplete. For now, the following limitation applies:
    ///
    /// * The full content of `source` is copied into memory.
    fn upload<R>(&self, source: &mut R, target: PutObjectRequest) -> S4Result<PutObjectOutput>
    where
        R: Read;

    /// Read `source` and upload it to S3 using multi-part upload
    ///
    /// # Caveats
    ///
    /// The current implementation is incomplete. For now, the following limitation applies:
    ///
    /// * The full content of a part is copied into memory.
    fn upload_multipart<R>(
        &self,
        source: &mut R,
        target: &PutObjectRequest,
        part_size: usize,
    ) -> S4Result<CompleteMultipartUploadOutput>
    where
        R: Read;

    /// Iterator over all objects
    ///
    /// Objects are lexicographically sorted by their key.
    fn iter_objects(&self, bucket: &str) -> ObjectIter;

    /// Iterator over objects with given `prefix`
    ///
    /// Objects are lexicographically sorted by their key.
    fn iter_objects_with_prefix(&self, bucket: &str, prefix: &str) -> ObjectIter;

    /// Iterator over all objects; fetching objects as needed
    ///
    /// Objects are lexicographically sorted by their key.
    fn iter_get_objects(&self, bucket: &str) -> GetObjectIter;

    /// Iterator over objects with given `prefix`; fetching objects as needed
    ///
    /// Objects are lexicographically sorted by their key.
    fn iter_get_objects_with_prefix(&self, bucket: &str, prefix: &str) -> GetObjectIter;
}

impl<'a> S4 for S3Client {
    fn download_to_file<F>(
        &self,
        source: GetObjectRequest,
        target: F,
    ) -> Result<GetObjectOutput, S4Error>
    where
        F: AsRef<Path>,
    {
        debug!("downloading to file {:?}", target.as_ref());
        let mut resp = self.get_object(source).sync()?;
        let mut body = resp.body.take().expect("no body");
        let mut target = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
        copy(&mut body, &mut target)?;
        Ok(resp)
    }

    #[inline]
    fn upload_from_file<F>(&self, source: F, target: PutObjectRequest) -> S4Result<PutObjectOutput>
    where
        F: AsRef<Path>,
    {
        debug!("uploading file {:?}", source.as_ref());
        let mut source = File::open(source)?;
        upload::upload(&self, &mut source, target)
    }

    #[inline]
    fn upload_from_file_multipart<F>(
        &self,
        source: F,
        target: &PutObjectRequest,
        part_size: usize,
    ) -> S4Result<CompleteMultipartUploadOutput>
    where
        F: AsRef<Path>,
    {
        debug!("uploading file {:?}", source.as_ref());
        let mut source = File::open(source)?;
        upload::upload_multipart(&self, &mut source, target, part_size)
    }

    fn download<W>(&self, source: GetObjectRequest, mut target: &mut W) -> S4Result<GetObjectOutput>
    where
        W: Write,
    {
        let mut resp = self.get_object(source).sync()?;
        let mut body = resp.body.take().expect("no body");
        copy(&mut body, &mut target)?;
        Ok(resp)
    }

    #[inline]
    fn upload<R>(&self, source: &mut R, target: PutObjectRequest) -> S4Result<PutObjectOutput>
    where
        R: Read,
    {
        upload::upload(&self, source, target)
    }

    #[inline]
    fn upload_multipart<R>(
        &self,
        mut source: &mut R,
        target: &PutObjectRequest,
        part_size: usize,
    ) -> S4Result<CompleteMultipartUploadOutput>
    where
        R: Read,
    {
        upload::upload_multipart(&self, &mut source, target, part_size)
    }

    #[inline]
    fn iter_objects(&self, bucket: &str) -> ObjectIter {
        ObjectIter::new(self, bucket, None)
    }

    #[inline]
    fn iter_objects_with_prefix(&self, bucket: &str, prefix: &str) -> ObjectIter {
        ObjectIter::new(self, bucket, Some(prefix))
    }

    #[inline]
    fn iter_get_objects(&self, bucket: &str) -> GetObjectIter {
        GetObjectIter::new(self, bucket, None)
    }

    #[inline]
    fn iter_get_objects_with_prefix(&self, bucket: &str, prefix: &str) -> GetObjectIter {
        GetObjectIter::new(self, bucket, Some(prefix))
    }
}

fn copy<W>(src: &mut StreamingBody, dest: &mut W) -> S4Result<()>
where
    W: Write,
{
    let src = src.take(512 * 1024).wait();
    for chunk in src {
        dest.write_all(chunk?.as_mut_slice())?;
    }
    Ok(())
}
