use futures::future;
use futures::{Future, Stream};

use actix_web::{
    dev, error, multipart, Error, FutureResponse, HttpMessage, HttpRequest, HttpResponse,
};

use std::fs;
use std::io::Write;

use crate::State;

/// from payload, save file
pub fn save_file(field: multipart::Field<dev::Payload>) -> Box<Future<Item = i64, Error = Error>> {
    use std::ffi::OsStr;
    use std::path::Path;

    let filename = field
        .content_disposition()
        .and_then(|cd| cd.get_filename().map(|st| st.to_string()))
        .unwrap_or("upload".to_string());
    let fileext = Path::new(&filename)
        .extension()
        .and_then(OsStr::to_str)
        .map_or("".to_string(), |ext| format!(".{}", ext));

    println!(
        "Saving file: filename {:?}; extension: {:?}",
        filename, fileext
    );
    let file_path_string = format!("./static/uploads/{}", filename);
    let mut file = match fs::File::create(file_path_string) {
        Ok(file) => file,
        Err(e) => return Box::new(future::err(error::ErrorInternalServerError(e))),
    };
    Box::new(
        field
            .fold(0i64, move |acc, bytes| {
                let rt = file
                    .write_all(bytes.as_ref())
                    .map(|_| acc + bytes.len() as i64)
                    .map_err(|e| {
                        println!("file.write_all failed: {:?}", e);
                        error::MultipartError::Payload(error::PayloadError::Io(e))
                    });
                future::result(rt)
            })
            .map_err(|e| {
                println!("save_file failed, {:?}", e);
                error::ErrorInternalServerError(e)
            }),
    )
}

pub fn handle_multipart_item(
    item: multipart::MultipartItem<dev::Payload>,
) -> Box<Stream<Item = i64, Error = Error>> {
    match item {
        multipart::MultipartItem::Field(field) => Box::new(save_file(field).into_stream()),
        multipart::MultipartItem::Nested(mp) => Box::new(
            mp.map_err(error::ErrorInternalServerError)
                .map(handle_multipart_item)
                .flatten(),
        ),
    }
}

pub fn upload(req: HttpRequest<State>) -> FutureResponse<HttpResponse> {
    use actix::Addr;
    use actix_web::error;

    use crate::sessions::UserSession;
    use crate::{is_signed_in_guard, SigninState};

    Box::new(
        is_signed_in_guard(&req)
            .and_then(|state| match state {
                SigninState::Valid(session) => Ok(session),
                _ => Err(error::ErrorForbidden("Must log in to upload")),
            })
            .and_then(move |session: UserSession| {
                req.multipart()
                    .map_err(error::ErrorInternalServerError)
                    .map(handle_multipart_item)
                    .flatten()
                    .collect()
                    .map(|sizes| HttpResponse::Ok().json(sizes))
                    .map_err(|e| {
                        println!("failed: {}", e);
                        e
                    })
            }),
    )
}
