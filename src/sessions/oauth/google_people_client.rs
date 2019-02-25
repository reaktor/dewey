extern crate reqwest;

use super::google_oauth::GoogleAccessToken;

use actix_web::client;
use actix_web::HttpMessage;
use actix_web::{error, FutureResponse};
use futures::Future;
use futures::future;

#[derive(Deserialize, Debug)]
struct GooglePeopleFieldMetadataSource {
    #[serde(rename = "type")]
    type_: Option<String>, // PROFILE, DOMAIN_PROFILE
    id: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GooglePeopleFieldMetadata {
    primary: Option<bool>,
    verified: Option<bool>, // for email addresses
    source: GooglePeopleFieldMetadataSource,
}

// getting profile information
// https://developers.google.com/people/api/rest/v1/people/get
#[derive(Deserialize, Debug)]
struct GooglePeopleName {
    metadata: GooglePeopleFieldMetadata, // "1/fFAGRNJru1FTz70BzhT3Zg",
    #[serde(rename = "displayName")]
    display_name: Option<String>, // "Cole Lawrence",
    #[serde(rename = "familyName")]
    family_name: Option<String>, // "Lawrence",
    #[serde(rename = "givenName")]
    given_name: Option<String>, // "Cole",
    #[serde(rename = "displayNameLastFirst")]
    display_name_last_first: Option<String>, // "Cole Lawrence"
}

#[derive(Deserialize, Debug)]
struct GooglePeopleEmailAddress {
    metadata: GooglePeopleFieldMetadata, // "1/fFAGRNJru1FTz70BzhT3Zg",
    value: String,                       // "Cole Lawrence",
}

#[derive(Deserialize, Debug)]
struct GooglePeopleResource {
    #[serde(rename = "resourceName")]
    resource_name: Option<String>,
    names: Option<Vec<GooglePeopleName>>,

    // TODO: in future be able to identify attendees by emails assoc to users?
    // emailAddresses: Option<Vec<GooglePeopleEmailAddress>>,
    error: Option<GooglePeopleError>,
}

#[derive(Deserialize, Debug)]
struct GooglePeopleError {
    // errors: Option<Vec<String>>
    message: String,
}

#[derive(Debug)]
pub struct IAm {
    pub resource_name: String,
    pub name: Option<String>,
}

pub fn who_am_i(access_token: &GoogleAccessToken) -> Result<IAm, String> {
    // https://people.googleapis.com/v1/{resourceName=people/*}
    let person_fields = "names"; // "names,emailAddresses"
    let url = format!(
        "https://people.googleapis.com/v1/people/me?personFields={}&access_token={}",
        person_fields, &access_token.access_token
    );
    let data: GooglePeopleResource = reqwest::get(&url)
        .map_err(|e| e.to_string())?
        .json::<GooglePeopleResource>()
        .map_err(|e| e.to_string())?;
    // let data: String = reqwest::get(&url).map_err(|e| e.to_string())?.text().map_err(|e| e.to_string())?;
    // Err(data)

    match data.resource_name {
        Some(res_name) => Ok(IAm {
            resource_name: res_name,
            name: data.names.and_then(|mut names| {
                names
                    .pop()
                    .and_then(|name0| name0.display_name.or(name0.given_name))
            }),
        }),
        None => match data.error {
            Some(err) => Err(format!("error: {:?}", err.message)),
            None => Err("no resourceName or error present".to_string()),
        },
    }
}

pub fn who_am_i_async(access_token: &GoogleAccessToken) -> FutureResponse<IAm> {
    info!("who am i async");
    // https://people.googleapis.com/v1/{resourceName=people/*}
    let person_fields = "names"; // "names,emailAddresses"
    let url = format!(
        "https://people.googleapis.com/v1/people/me?personFields={}&access_token={}",
        person_fields, &access_token.access_token
    );

    Box::new(
        client::get(&url)
            .header("User-Agent", "Actix-web")
            .header("Accept-Encoding", "identity")
            .finish()
            .unwrap()
            .send()
            .timeout(std::time::Duration::from_secs(10))
            .map_err(|e| {
                warn!("Failed to send WhoAmI for GooglePeopleResource {:?}", e);
                error::ErrorInternalServerError("Who am I send error")
            })
            .and_then(|resp: actix_web::client::ClientResponse| {
                if resp.status().is_success() {
                    future::Either::A(resp.json::<GooglePeopleResource>().map_err(|e| {
                        warn!("Failed to parse GooglePeopleResource {:?}", e);
                        error::ErrorInternalServerError("Who am I json parse error")
                    }))
                } else {
                    future::Either::B(future::err(error::ErrorBadRequest(format!(
                        "Who am I request error [{}], please try again",
                        resp.status()
                    ))))
                }
            })
            .and_then(move |data: GooglePeopleResource| match data.resource_name {
                Some(res_name) => Ok(IAm {
                    resource_name: res_name,
                    name: data.names.and_then(|mut names| {
                        names
                            .pop()
                            .and_then(|name0| name0.display_name.or(name0.given_name))
                    }),
                }),
                None => Err(match data.error {
                    Some(err) => {
                        error::ErrorInternalServerError(format!("error: {:?}", err.message))
                    }
                    None => error::ErrorInternalServerError(
                        "no resourceName or error present".to_string(),
                    ),
                }),
            }),
    )
}
