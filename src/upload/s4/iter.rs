//! Iterators over `Object`s
//!
//! # Example
//!
//! ```
//! extern crate fallible_iterator;
//! extern crate futures;
//! extern crate rand;
//! extern crate rusoto_core;
//! extern crate rusoto_s3;
//! extern crate s4;
//!
//! use fallible_iterator::FallibleIterator;
//! use futures::stream::Stream;
//! use futures::Future;
//! use rand::RngCore;
//! use rusoto_core::Region;
//! use rusoto_s3::{CreateBucketRequest, PutObjectRequest, S3, S3Client};
//! use s4::S4;
//! use std::env;
//!
//! fn main() {
//!     let bucket = format!("iter-module-example-{}", rand::thread_rng().next_u64());
//!
//!     // setup client
//!
//!     let access_key = "ANTN35UAENTS5UIAEATD".to_string();
//!     let secret_key = "TtnuieannGt2rGuie2t8Tt7urarg5nauedRndrur".to_string();
//!     let endpoint = env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
//!     let region = Region::Custom {
//!         name: "eu-west-1".to_string(),
//!         endpoint,
//!     };
//!     let client = s4::new_s3client_with_credentials(region, access_key, secret_key).unwrap();
//!
//!     // create bucket
//!
//!     client
//!         .create_bucket(CreateBucketRequest {
//!             bucket: bucket.clone(),
//!             ..Default::default()
//!         })
//!         .sync()
//!         .expect("failed to create bucket");
//!
//!     // create test objects
//!
//!     for obj in (0..5).map(|n| format!("object_{:02}", n)) {
//!         client
//!             .put_object(PutObjectRequest {
//!                 bucket: bucket.clone(),
//!                 key: obj.to_string(),
//!                 body: Some(obj.as_bytes().to_vec().into()),
//!                 ..Default::default()
//!             })
//!             .sync()
//!             .expect("failed to store object");
//!     }
//!
//!     // iterate over objects objects (sorted alphabetically)
//!
//!     let objects: Vec<_> = client
//!         .iter_objects(&bucket)
//!         .map(|obj| obj.key.unwrap())
//!         .collect()
//!         .unwrap();
//!
//!     assert_eq!(
//!         objects.as_slice(),
//!         &[
//!             "object_00",
//!             "object_01",
//!             "object_02",
//!             "object_03",
//!             "object_04",
//!         ]
//!     );
//!
//!     // iterate object and fetch content on the fly (sorted alphabetically)
//!
//!     let objects: Vec<(String, Vec<u8>)> = client
//!         .iter_get_objects(&bucket)
//!         .map(|(key, obj)| (key, obj.body.unwrap().concat2().wait().unwrap()))
//!         .collect()
//!         .expect("failed to fetch content");
//!
//!     for (i, (key, body)) in objects.iter().enumerate() {
//!         let expected = format!("object_{:02}", i);
//!         assert_eq!(key, &expected);
//!         assert_eq!(body.as_slice(), expected.as_bytes());
//!     }
//! }
//! ```

use super::error::{S4Error, S4Result};
use fallible_iterator::FallibleIterator;
use rusoto_s3::{
    GetObjectOutput, GetObjectRequest, ListObjectsV2Error, ListObjectsV2Request, Object, S3Client,
    S3,
};
use std::mem;
use std::vec::IntoIter;

/// Iterator over all objects or objects with a given prefix
pub struct ObjectIter<'a> {
    client: &'a S3Client,
    request: ListObjectsV2Request,
    objects: IntoIter<Object>,
    exhausted: bool,
}

impl<'a> Clone for ObjectIter<'a> {
    fn clone(&self) -> Self {
        ObjectIter {
            client: self.client,
            request: self.request.clone(),
            objects: self.objects.clone(),
            exhausted: self.exhausted,
        }
    }
}

impl<'a> ObjectIter<'a> {
    pub fn new(client: &'a S3Client, bucket: &str, prefix: Option<&str>) -> Self {
        let request = ListObjectsV2Request {
            bucket: bucket.to_owned(),
            max_keys: Some(1000),
            prefix: prefix.map(|s| s.to_owned()),
            ..Default::default()
        };

        ObjectIter {
            client,
            request,
            objects: Vec::new().into_iter(),
            exhausted: false,
        }
    }

    fn next_objects(&mut self) -> Result<(), ListObjectsV2Error> {
        let resp = self.client.list_objects_v2(self.request.clone()).sync()?;
        self.objects = resp.contents.unwrap_or_else(Vec::new).into_iter();
        match resp.next_continuation_token {
            next @ Some(_) => self.request.continuation_token = next,
            None => self.exhausted = true,
        };
        Ok(())
    }

    fn last_internal(&mut self) -> Result<Option<Object>, ListObjectsV2Error> {
        let mut objects = mem::replace(&mut self.objects, Vec::new().into_iter());
        while !self.exhausted {
            self.next_objects()?;
            if self.objects.len() > 0 {
                objects = mem::replace(&mut self.objects, Vec::new().into_iter());
            }
        }
        Ok(objects.last())
    }
}

impl<'a> FallibleIterator for ObjectIter<'a> {
    type Item = Object;
    type Error = ListObjectsV2Error;

    fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
        if let object @ Some(_) = self.objects.next() {
            Ok(object)
        } else if self.exhausted {
            Ok(None)
        } else {
            self.next_objects()?;
            Ok(self.objects.next())
        }
    }

    fn count(mut self) -> Result<usize, Self::Error> {
        let mut count = self.objects.len();
        while !self.exhausted {
            self.next_objects()?;
            count += self.objects.len();
        }
        Ok(count)
    }

    #[inline]
    fn last(mut self) -> Result<Option<Self::Item>, Self::Error> {
        self.last_internal()
    }

    fn nth(&mut self, mut n: usize) -> Result<Option<Self::Item>, Self::Error> {
        while self.objects.len() <= n && !self.exhausted {
            n -= self.objects.len();
            self.next_objects()?;
        }
        Ok(self.objects.nth(n))
    }
}

/// Iterator retrieving all objects or objects with a given prefix
///
/// The iterator yields tuples of `(key, object)`.
pub struct GetObjectIter<'a> {
    inner: ObjectIter<'a>,
    request: GetObjectRequest,
}

impl<'a> Clone for GetObjectIter<'a> {
    fn clone(&self) -> Self {
        GetObjectIter {
            inner: self.inner.clone(),
            request: self.request.clone(),
        }
    }
}

impl<'a> GetObjectIter<'a> {
    pub fn new(client: &'a S3Client, bucket: &str, prefix: Option<&str>) -> Self {
        let request = GetObjectRequest {
            bucket: bucket.to_owned(),
            ..Default::default()
        };

        GetObjectIter {
            inner: ObjectIter::new(client, bucket, prefix),
            request,
        }
    }

    fn retrieve(&mut self, object: Option<Object>) -> S4Result<Option<(String, GetObjectOutput)>> {
        match object {
            Some(object) => {
                self.request.key = object
                    .key
                    .ok_or_else(|| S4Error::Other("response is missing key"))?;
                match self.inner.client.get_object(self.request.clone()).sync() {
                    Ok(o) => {
                        let key = mem::replace(&mut self.request.key, String::new());
                        Ok(Some((key, o)))
                    }
                    Err(e) => Err(e.into()),
                }
            }
            None => Ok(None),
        }
    }
}

impl<'a> FallibleIterator for GetObjectIter<'a> {
    type Item = (String, GetObjectOutput);
    type Error = S4Error;

    #[inline]
    fn next(&mut self) -> S4Result<Option<Self::Item>> {
        let next = self.inner.next()?;
        self.retrieve(next)
    }

    #[inline]
    fn count(self) -> Result<usize, Self::Error> {
        self.inner.count().map_err(|e| e.into())
    }

    #[inline]
    fn last(mut self) -> Result<Option<Self::Item>, Self::Error> {
        let last = self.inner.last_internal()?;
        self.retrieve(last)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Result<Option<Self::Item>, Self::Error> {
        let nth = self.inner.nth(n)?;
        self.retrieve(nth)
    }
}
