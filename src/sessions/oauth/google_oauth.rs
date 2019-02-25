use actix_web::client;
use actix_web::HttpMessage;
use actix_web::{error, FutureResponse};
use futures::future;
use futures::Future;

use chrono::{DateTime, Utc, Duration};

#[derive(Clone, Debug)]
pub struct GoogleAccessToken {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}


#[derive(Deserialize, Debug)]
struct GoogleTokenAuthCodeJson {
    // success
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: Option<String>,
    // error
    pub error: Option<String>,
    pub error_description: Option<String>,
}

#[derive(Debug)]
pub enum ExchangeResult {
    AccessTokenOnly(GoogleAccessToken),
    AccessAndRefreshTokens {
        access: GoogleAccessToken,
        refresh: String,
    },
}

pub fn exchange_code_for_token(code: &str) -> FutureResponse<ExchangeResult> {
    info!("exchange_code_for_token");
    // Construct a request against http://localhost:8020/token, the access token endpoint
    let redirect_uri = format!("{}/login/google/callback", dotenv!("ROOT_HOST"));

    let client_id = dotenv!("GOOGLE_OAUTH_CLIENT_ID");
    let client_secret = dotenv!("GOOGLE_OAUTH_CLIENT_SECRET");
    let google_token_endpoint = "https://www.googleapis.com/oauth2/v4/token";

    // https://developers.google.com/identity/protocols/OAuth2WebServer#offline
    let params = [
        ("code", code.as_ref()),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("redirect_uri", &redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    // Not sure why "Accept-Encoding" "identity" works to make it resolve far more quickly
    // https://github.com/actix/actix-web/issues/674#issuecomment-466720953

    Box::new(
        client::post(google_token_endpoint)
            .header("User-Agent", "Actix-web")
            .header("Accept-Encoding", "identity")
            .form(&params)
            .unwrap()
            .send()
            .timeout(std::time::Duration::from_secs(10))
            .map_err(|e| {
                warn!("Failed to send code params for Token exchange: {:?}", e);
                error::ErrorInternalServerError("Code exchange send error")
            })
            .and_then(|resp: actix_web::client::ClientResponse| {
                info!("exchange_code_for_token client response json {:?}", resp);
                if resp.status().is_success() {
                    future::Either::A(resp.json::<GoogleTokenAuthCodeJson>().map_err(|e| {
                        warn!("Failed to parse GoogleTokenAuthCodeJson {:?}", e);
                        error::ErrorInternalServerError("Code exchange json parse error")
                    }))
                } else {
                    future::Either::B(future::err(error::ErrorBadRequest(format!(
                        "Code exchange request error [{}], please try again",
                        resp.status()
                    ))))
                }
            })
            .and_then(move |token_map: GoogleTokenAuthCodeJson| {
                info!("exchange_code_for_token token_map matching");
                match (token_map.access_token, token_map.expires_in) {
                    (Some(access), Some(expires_in)) => {
                        let expires_at = Utc::now() + Duration::seconds(expires_in);
                        let access_token = GoogleAccessToken {
                            access_token: access,
                            expires_at,
                        };
                        Ok(match token_map.refresh_token {
                            Some(refresh) => ExchangeResult::AccessAndRefreshTokens {
                                access: access_token,
                                refresh,
                            },
                            None => ExchangeResult::AccessTokenOnly(access_token),
                        })
                    }
                    _ => Err(error::ErrorInternalServerError(format!(
                        "Error with received tokens: {}",
                        token_map
                            .error
                            .or(token_map.error_description)
                            .unwrap_or("Access token missing".to_string())
                    ))),
                }
            }),
    )
}

// Refreshing a token
// https://developers.google.com/identity/protocols/OAuth2WebServer#offline
#[derive(Deserialize, Debug)]
struct GoogleTokenRefresh {
    pub access_token: String, // "1/fFAGRNJru1FTz70BzhT3Zg",
    pub expires_in: i64,      //  3920,
    pub token_type: String,   // "Bearer"
}

pub fn refresh_google_token(refresh_token: &str) -> FutureResponse<GoogleAccessToken> {
    let client_id = dotenv!("GOOGLE_OAUTH_CLIENT_ID");
    let client_secret = dotenv!("GOOGLE_OAUTH_CLIENT_SECRET");
    let google_token_endpoint = "https://www.googleapis.com/oauth2/v4/token";
    // let google_token_endpoint = "http://httpbin.org/post";

    // https://developers.google.com/identity/protocols/OAuth2WebServer#offline
    let params = [
        ("refresh_token", refresh_token),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "refresh_token"),
    ];

    Box::new(
        client::post(google_token_endpoint)
            .header("User-Agent", "Actix-web")
            .header("Accept-Encoding", "identity")
            .form(&params)
            .unwrap()
            .send()
            .timeout(std::time::Duration::from_secs(10))
            .map_err(|e| {
                warn!("Failed to send refresh token for Token refresh: {:?}", e);
                error::ErrorInternalServerError("Token refresh send error")
            })
            .and_then(|resp: actix_web::client::ClientResponse| {
                if resp.status().is_success() {
                    future::Either::A(resp.json::<GoogleTokenRefresh>().map_err(|e| {
                        warn!("Failed to parse GoogleTokenAuthCodeJson {:?}", e);
                        error::ErrorInternalServerError("Token refresh json parse error")
                    }))
                } else {
                    future::Either::B(future::err(error::ErrorBadRequest(format!(
                        "Token refresh request error [{}], please try again",
                        resp.status()
                    ))))
                }
            })
            .map(move |resp_json: GoogleTokenRefresh| GoogleAccessToken {
                access_token: resp_json.access_token,
                expires_at: Utc::now() + Duration::seconds(resp_json.expires_in),
            }),
    )
}

pub fn revoke_token(token: &GoogleAccessToken) -> FutureResponse<()> {
    let google_token_endpoint = "https://accounts.google.com/o/oauth2/revoke";

    // https://developers.google.com/identity/protocols/OAuth2WebServer#offline
    let url = format!("{}?token={}", google_token_endpoint, &token.access_token);
    Box::new(
        client::get(&url)
            .header("User-Agent", "Actix-web")
            .header("Accept-Encoding", "identity")
            .finish()
            .unwrap()
            .send()
            .timeout(std::time::Duration::from_secs(10))
            .map_err(|e| {
                warn!("Error revoking token: {:?}", e);
                error::ErrorInternalServerError("Error revoking token")
            })
            .map(|_| {
                info!("Successfully revoked user's tokens");
                ()
            }),
    )
}
