use crate::utils::build_request_headers;

#[cfg(feature = "login")]
pub mod login;
mod utils;

pub const PINTEREST_BASE_URL: &str = "https://www.pinterest.com";
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
            Chrome/91.0.4472.114 Safari/537.36";


pub struct Pinterest {
    client: reqwest::Client,
    username: String,
    #[cfg(feature = "credentials")]
    crf_token: String,
    #[cfg(feature = "login")]
    user_agent: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PinterestError {
    #[cfg(feature = "login")]
    #[error("{0}")]
    LoginError(#[from] pinterest_login::PinterestLoginError),
    #[cfg(feature = "login")]
    #[error("The crf token was not found in the cookies")]
    MissingCrfToken,
    /*#[error("There was an error parsing the url: {0}")]
    UrlError(#[from] url::ParseError),*/
    #[cfg(feature = "login")]
    #[error("There was an error reading the credentials file: {0}")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "login")]
    #[error("There was an error deserializing the credentials file")]
    InvalidCredentialsFile,
    #[cfg(feature = "login")]
    #[error("The credentials file path does not exist")]
    CredPathNotExists,
    #[error("There was an error sending the request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("There was an error parsing the response: {0}")]
    ResponseError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PinterestError>;


impl Pinterest {
    pub fn new<S: Into<String> + Copy>(
        username: S,
        #[cfg(feature = "credentials")] crf_token: S,
        user_agent: Option<S>,
    ) -> Self {
        #[cfg(feature = "credentials")]
            let header_map = build_request_headers(crf_token.into(), user_agent.map(|s| s.into()));

        #[cfg(feature = "credentials")]
            let client = reqwest::Client::builder()
            .default_headers(header_map)
            .build().expect("Failed to build request client");

        #[cfg(not(feature = "credentials"))]
            let client = reqwest::Client::new();

        Self {
            client,
            username: username.into(),
            #[cfg(feature = "credentials")]
            crf_token: crf_token.into(),
            #[cfg(feature = "login")]
            user_agent: user_agent.map(|s| s.into()),
        }
    }
}
