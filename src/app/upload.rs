use futures::{future, Future, IntoFuture};
use ring::digest;

use actix_web::{error, http, FutureResponse, HttpMessage, HttpRequest, HttpResponse};

use crate::user::User;
use crate::State;

#[derive(Serialize)]
struct URL {
    url: String,
    key: String,
}

pub fn get_url(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
    use crate::object::store;
    use crate::object::ObjectStore;
    use actix::Addr;
    use actix_web::error;
    let store_actor: Addr<ObjectStore> = req.state().store.clone();

    use crate::sessions::UserSession;
    use crate::{is_signed_in_guard, SigninState};

    Box::new(
        is_signed_in_guard(&req)
            .and_then(|state| match state {
                SigninState::Valid(session) => Ok(session),
                _ => Err(error::ErrorForbidden("Must log in to upload")),
            })
            .and_then(move |session: UserSession| {
                store_actor
                    .send(store::GetPendingPutUrl {
                        user_id: session.person.user_id,
                    })
                    .map_err(|_| error::ErrorBadRequest("Failed to get put url"))
                    .and_then(|res| res)
                    .map_err(|_| error::ErrorBadRequest("Failed to get put url"))
            })
            .map(|put_url: store::PendingPutUrl| {
                HttpResponse::Ok()
                    .header(actix_web::http::header::CONTENT_TYPE, "application/json")
                    .json(URL {
                        url: put_url.url,
                        key: put_url.key,
                    })
            }),
    )
}

#[derive(Deserialize)]
struct PostCompleteRequestJSON {
    key: String,
}

pub fn post_complete(req: &HttpRequest<State>) -> FutureResponse<HttpResponse> {
    use crate::object::store;
    use crate::object::ObjectStore;
    use actix::Addr;
    use actix_web::error;
    let store_actor: Addr<ObjectStore> = req.state().store.clone();
    use crate::sessions::UserSession;
    use crate::{is_signed_in_guard, SigninState};

    Box::new(
        is_signed_in_guard(&req)
            .and_then(|state| match state {
                SigninState::Valid(session) => Ok(session),
                _ => Err(error::ErrorForbidden("Must log in to upload")),
            })
            .join(req.json::<PostCompleteRequestJSON>().map_err(|a| a.into()))
            .and_then(
                move |(session, req): (UserSession, PostCompleteRequestJSON)| {
                    ensure_ownership_of_pending(&session.person, &req.key)
                        .into_future()
                        .and_then(move |_| {
                            store_actor
                                .send(store::FinalizeObject { key: req.key })
                                .map_err(|_| error::ErrorBadRequest("Failed to finalize object"))
                                .and_then(|res| res)
                                .map_err(|_| error::ErrorBadRequest("Failed to finalize object"))
                                .map(move |stored_object: store::StoredObject| {
                                    info!(
                                        "Uploaded and retrieved: {} => {}",
                                        session.person.user_id, stored_object.key
                                    );
                                    HttpResponse::Ok()
                                        .header(
                                            actix_web::http::header::CONTENT_TYPE,
                                            "application/json",
                                        )
                                        .json(URL {
                                            url: stored_object.bucket,
                                            key: stored_object.key,
                                        })
                                })
                        })
                },
            ),
    )
}

/// API calls related to upload functionality
pub fn upload_scope(scope: actix_web::Scope<State>) -> actix_web::Scope<State> {
    scope
        .resource("/url", |r| r.method(http::Method::GET).f(get_url))
        .resource("/complete", |r| {
            r.method(http::Method::POST).f(post_complete)
        })
}

fn ensure_ownership_of_pending<U: User>(user: &U, key: &str) -> Result<(), error::Error> {
    let user_id_str = format!("{}", user.id());
    if key.contains(&user_id_str) {
        Ok(())
    } else {
        Err(error::ErrorUnauthorized(
            "Needs ownership of pending object",
        ))
    }
}
