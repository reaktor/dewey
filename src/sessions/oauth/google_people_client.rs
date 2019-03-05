use super::google_oauth::GoogleAccessToken;

use actix_web::client;
use actix_web::HttpMessage;
use actix_web::{error, FutureResponse};
use futures::future;
use futures::Future;

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

// getting profile information
// https://developers.google.com/people/api/rest/v1/people/get
#[derive(Deserialize, Debug)]
struct GooglePeoplePhoto {
    metadata: GooglePeopleFieldMetadata, // "1/fFAGRNJru1FTz70BzhT3Zg",
    url: Option<String>, // "https://lh3.googleusercontent.com/-XhSIc7llbfY/AAAAAAAAAAI/AAAAAAAAAHo/O6vIYycBkTw/s100/photo.jpg",
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
    #[serde(rename = "emailAddresses")]
    email_addresses: Option<Vec<GooglePeopleEmailAddress>>,
    photos: Option<Vec<GooglePeoplePhoto>>,
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
    pub given_name: String,
    pub display_name: String,
    pub email_address: String,
    pub photo_url: String,
}

pub fn who_am_i_async(access_token: &GoogleAccessToken) -> FutureResponse<IAm> {
    info!("who am i async");
    // https://people.googleapis.com/v1/{resourceName=people/*}
    let person_fields = "names,emailAddresses,photos";
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
            .and_then(move |data: GooglePeopleResource| {
                info!("Successfully retrieved IAm => {:?}", data);

                let name0 = data.names.and_then(|mut names| names.pop());
                let email0: String = data
                    .email_addresses
                    .and_then(|mut em| em.pop())
                    .map(|em: GooglePeopleEmailAddress| em.value)
                    .ok_or(error::ErrorInternalServerError(
                        "Email address needs to be present on account".to_string(),
                    ))?;
                let photo0: String = data
                    .photos
                    .and_then(|mut em| em.pop())
                    .and_then(|em: GooglePeoplePhoto| em.url)
                    .ok_or(error::ErrorInternalServerError(
                        "Photo needs to be present on account".to_string(),
                    ))?;

                match data.resource_name {
                    Some(res_name) => Ok(IAm {
                        given_name: name0
                            .as_ref()
                            .and_then(|n| n.given_name.as_ref().or(n.display_name.as_ref()))
                            .unwrap_or(&email0)
                            .to_owned(),
                        display_name: name0
                            .as_ref()
                            .and_then(|n| n.display_name.as_ref().or(n.given_name.as_ref()))
                            .unwrap_or(&email0)
                            .to_owned(),
                        resource_name: res_name,
                        email_address: email0,
                        photo_url: photo0,
                    }),
                    None => Err(match data.error {
                        Some(err) => {
                            error::ErrorInternalServerError(format!("error: {:?}", err.message))
                        }
                        None => error::ErrorInternalServerError(
                            "no resourceName or error present".to_string(),
                        ),
                    }),
                }
            }),
    )
}
