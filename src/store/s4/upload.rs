use super::error::{S4Error, S4Result};
use rusoto_s3::{
    AbortMultipartUploadRequest, CompleteMultipartUploadOutput, CompleteMultipartUploadRequest,
    CompletedMultipartUpload, CompletedPart, CreateMultipartUploadRequest, PutObjectOutput,
    PutObjectRequest, S3Client, UploadPartRequest, S3,
};
use std::io::Read;

pub fn upload<R>(
    client: &S3Client,
    source: &mut R,
    mut target: PutObjectRequest,
) -> S4Result<PutObjectOutput>
where
    R: Read,
{
    let mut content = Vec::new();
    source.read_to_end(&mut content)?;
    target.body = Some(content.into());
    client.put_object(target).sync().map_err(|e| e.into())
}

pub fn upload_multipart<R>(
    client: &S3Client,
    source: &mut R,
    target: &PutObjectRequest,
    part_size: usize,
) -> S4Result<CompleteMultipartUploadOutput>
where
    R: Read,
{
    let upload = client
        .create_multipart_upload(CreateMultipartUploadRequest {
            acl: target.acl.to_owned(),
            bucket: target.bucket.to_owned(),
            cache_control: target.cache_control.to_owned(),
            content_disposition: target.content_disposition.to_owned(),
            content_encoding: target.content_encoding.to_owned(),
            content_language: target.content_language.to_owned(),
            content_type: target.content_type.to_owned(),
            expires: target.expires.to_owned(),
            grant_full_control: target.grant_full_control.to_owned(),
            grant_read: target.grant_read.to_owned(),
            grant_read_acp: target.grant_read_acp.to_owned(),
            grant_write_acp: target.grant_write_acp.to_owned(),
            key: target.key.to_owned(),
            metadata: target.metadata.to_owned(),
            request_payer: target.request_payer.to_owned(),
            sse_customer_algorithm: target.sse_customer_algorithm.to_owned(),
            sse_customer_key: target.sse_customer_key.to_owned(),
            sse_customer_key_md5: target.sse_customer_key_md5.to_owned(),
            ssekms_key_id: target.ssekms_key_id.to_owned(),
            server_side_encryption: target.server_side_encryption.to_owned(),
            storage_class: target.storage_class.to_owned(),
            tagging: target.tagging.to_owned(),
            website_redirect_location: target.website_redirect_location.to_owned(),
        })
        .sync()?;

    let upload_id = upload
        .upload_id
        .ok_or_else(|| S4Error::Other("Missing upload ID"))?;

    debug!(
        "multi-part upload {:?} started (bucket: {}, key: {})",
        upload_id, target.bucket, target.key
    );

    match upload_multipart_needs_abort_on_error(&client, source, target, part_size, &upload_id) {
        ok @ Ok(_) => ok,
        err @ Err(_) => {
            info!(
                "aborting upload {:?} due to a failure during upload",
                upload_id
            );
            if let Err(e) = client
                .abort_multipart_upload(AbortMultipartUploadRequest {
                    bucket: target.bucket.to_owned(),
                    key: target.key.to_owned(),
                    request_payer: target.request_payer.to_owned(),
                    upload_id,
                })
                .sync()
            {
                warn!("ignoring failure to abort multi-part upload: {:?}", e);
            };
            err
        }
    }
}

// Upload needs to be aborted if this function fails
fn upload_multipart_needs_abort_on_error<R>(
    client: &S3Client,
    source: &mut R,
    target: &PutObjectRequest,
    part_size: usize,
    upload_id: &str,
) -> S4Result<CompleteMultipartUploadOutput>
where
    R: Read,
{
    let mut parts = Vec::new();
    for part_number in 1.. {
        let mut body = vec![0; part_size];
        let size = source.read(&mut body[..])?;
        if size == 0 {
            break;
        }
        body.truncate(size);

        let part = client
            .upload_part(UploadPartRequest {
                body: Some(body.into()),
                bucket: target.bucket.clone(),
                content_length: None,
                content_md5: None,
                key: target.key.clone(),
                part_number,
                request_payer: target.request_payer.clone(),
                sse_customer_algorithm: target.sse_customer_algorithm.clone(),
                sse_customer_key: target.sse_customer_key.clone(),
                sse_customer_key_md5: target.sse_customer_key_md5.clone(),
                upload_id: upload_id.to_owned(),
            })
            .sync()?;

        parts.push(CompletedPart {
            e_tag: part.e_tag,
            part_number: Some(part_number),
        });
    }

    client
        .complete_multipart_upload(CompleteMultipartUploadRequest {
            bucket: target.bucket.to_owned(),
            key: target.key.to_owned(),
            multipart_upload: Some(CompletedMultipartUpload { parts: Some(parts) }),
            request_payer: target.request_payer.to_owned(),
            upload_id: upload_id.to_owned(),
        })
        .sync()
        .map_err(|e| e.into())
}
