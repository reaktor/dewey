#[derive(Debug)]
pub struct GoogleAccessToken {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct GoogleTokenAuthCodeJson {
    // success
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub token_type: Option<String>,
    // error
    pub error: Option<String>,
    pub error_description: Option<String>,
}

use chrono::{DateTime, Utc, Duration};

#[derive(Debug)]
pub enum ExchangeResult {
    AccessTokenOnly(GoogleAccessToken),
    AccessAndRefreshTokens {
        access: GoogleAccessToken,
        refresh: String,
    },
    GoogleError(String),
    FetchError(String),
    ParsingError(String),
}

pub fn exchange_code_for_token(code: &str) -> ExchangeResult {
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

    let client = reqwest::Client::new();
    let access_token_request = client
        .post(google_token_endpoint)
        .form(&params)
        .build()
        .unwrap();

    let mut token_response = match client.execute(access_token_request) {
        Ok(response) => response,
        Err(_) => return ExchangeResult::FetchError("Could not fetch bearer token".to_string()),
    };

    let token_map: GoogleTokenAuthCodeJson = match token_response.json() {
        Ok(token_map) => token_map,
        Err(err) => {
            return ExchangeResult::ParsingError(format!(
                "Error unwrapping json response, got {:?} instead",
                err
            ));
        }
    };

    match (token_map.access_token, token_map.expires_in) {
        (Some(access), Some(expires_in)) => {
            let expires_at = Utc::now() + Duration::seconds(expires_in);
            let access_token = GoogleAccessToken { access_token: access, expires_at };
            match token_map.refresh_token {
                Some(refresh) => ExchangeResult::AccessAndRefreshTokens {
                    access: access_token, refresh
                },
                None => ExchangeResult::AccessTokenOnly(access_token),
            }
        },
        _ => ExchangeResult::GoogleError(
            token_map
                .error
                .or(token_map.error_description)
                .unwrap_or("Access token missing".to_string()),
        )
    }
}



// Refreshing a token
// https://developers.google.com/identity/protocols/OAuth2WebServer#offline
#[derive(Deserialize, Debug)]
struct GoogleTokenRefresh {
    pub access_token: String, // "1/fFAGRNJru1FTz70BzhT3Zg",
    pub expires_in: i64,      //  3920,
    pub token_type: String,   // "Bearer"
}

pub fn refresh_google_token(refresh_token: &str) -> Result<GoogleAccessToken, String> {
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

    let client = reqwest::Client::new();
    trace!("Code {}", refresh_token);
    let mut res = client
        .post(google_token_endpoint)
        .form(&params)
        .send()
        .map_err(|e| e.to_string())?; // TODO: use failure crate for errors

    debug!("oauth refresh token resp - Status: {}", res.status());
    trace!("oauth refresh token resp - Headers:\n{:?}", res.headers());
    let resp_json = res.json::<GoogleTokenRefresh>().map_err(|e| e.to_string())?;
    debug!("oauth refresh token resp - {:?}", &resp_json);

    Ok(GoogleAccessToken {
        access_token: resp_json.access_token,
        expires_at: Utc::now() + Duration::seconds(resp_json.expires_in),
    })
}

pub fn revoke_token(token: &GoogleAccessToken) -> Result<(), String> {
    let google_token_endpoint = "https://accounts.google.com/o/oauth2/revoke";

    // https://developers.google.com/identity/protocols/OAuth2WebServer#offline
    let client = reqwest::Client::new();
    let url = format!("{}?token={}", google_token_endpoint, &token.access_token);
    client.get(&url).send().map_err(|e| e.to_string())?; // TODO: use failure crate for errors

    Ok(())
}
