use std::env;
use std::fmt;

#[derive(Clone, Debug)]
pub struct Configuration {
    database_url: String,
    google_oauth_client_id: String,
    google_oauth_client_secret: String,
    http_host: String,
    http_port: i32,
    redis_url: String,
    s3_access_key_id: String,
    s3_secret_access_key: String,
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration {
            database_url: String::from("postgres://localhost/collect"),
            google_oauth_client_id: String::from(""),
            google_oauth_client_secret: String::from(""),
            http_host: String::from("localhost"),
            http_port: 8088i32,
            redis_url: String::from("127.0.0.1:6379"),
            s3_access_key_id: String::from(""),
            s3_secret_access_key: String::from(""),
        }
    }

    pub fn http_host(&self) -> &str {
        self.http_host.as_ref()
    }

    pub fn http_port(&self) -> i32 {
        self.http_port
    }

    pub fn database_url(&self) -> &str {
        self.database_url.as_ref()
    }

    pub fn redis_url(&self) -> &str {
        self.redis_url.as_ref()
    }

    pub fn s3_access_key_id(&self) -> &str {
        self.s3_access_key_id.as_ref()
    }

    pub fn s3_secret_access_key(&self) -> &str {
        self.s3_secret_access_key.as_ref()
    }

    pub fn google_oauth_client_id(&self) -> &str {
        self.google_oauth_client_id.as_ref()
    }

    pub fn google_oauth_client_secret(&self) -> &str {
        self.google_oauth_client_secret.as_ref()
    }

    pub fn from_environment(mut self) -> Configuration {
        if let Ok(http_host) = env::var("ROOT_HOST") {
            self.http_host = http_host
        }
        if let Ok(http_port_s) = env::var("HTTP_PORT") {
            if let Ok(http_port) = http_port_s.parse::<i32>() {
                self.http_port = http_port
            }
        }
        if let Ok(database_url) = env::var("DATABASE_URL") {
            self.database_url = database_url
        }
        if let Ok(redis_url) = env::var("REDIS_URL") {
            self.redis_url = redis_url
        }
        if let Ok(s3_access_key_id) = env::var("S3_ACCESS_KEY_ID") {
            self.s3_access_key_id = s3_access_key_id
        }
        if let Ok(s3_secret_access_key) = env::var("S3_SECRET_ACCESS_KEY") {
            self.s3_secret_access_key = s3_secret_access_key
        }
        if let Ok(google_oauth_client_id) = env::var("GOOGLE_OAUTH_CLIENT_ID") {
            self.google_oauth_client_id = google_oauth_client_id
        }
        if let Ok(google_oauth_client_secret) = env::var("GOOGLE_OAUTH_CLIENT_SECRET") {
            self.google_oauth_client_secret = google_oauth_client_secret
        }
        self
    }

    pub fn from_arguments(
        mut self,
        arguments: &clap::ArgMatches,
    ) -> Result<Configuration, ArgumentError> {
        if let Some(port_s) = arguments.value_of("PORT") {
            if let Ok(port) = port_s.parse::<i32>() {
                self.http_port = port;
            } else {
                return Err(ArgumentError {
                    argument: "PORT",
                    expected: "must be an integer",
                });
            }
        }
        if let Some(host) = arguments.value_of("HOST") {
            self.http_host = String::from(host);
        }

        Ok(self)
    }
}

#[derive(Debug)]
pub struct ArgumentError {
    argument: &'static str,
    expected: &'static str,
}
impl fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} must be {}", self.argument, self.expected)
    }
}
