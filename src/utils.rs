use reqwest::header::{HeaderMap, HeaderValue};
use crate::{DEFAULT_USER_AGENT, PINTEREST_BASE_URL};

#[inline(always)]
pub(crate) fn build_request_headers(
    #[cfg(any(feature = "credentials", feature = "login"))] crf_token: String,
    user_agent: Option<String>) -> HeaderMap {
    let mut reqwest_headers: HeaderMap = HeaderMap::with_capacity(
        {
            #[cfg(any(feature = "credentials", feature = "login"))]
            { 5 }
            #[cfg(not(any(feature = "credentials", feature = "login")))]
            { 4 }
        }
    );
    reqwest_headers.insert(
        "User-Agent",
        HeaderValue::from_str(&user_agent.unwrap_or_else(|| DEFAULT_USER_AGENT.to_string())).unwrap(),
    );
    #[cfg(any(feature = "credentials", feature = "login"))]
    reqwest_headers.insert("X-CSRFToken", HeaderValue::from_str(&crf_token).unwrap());

    reqwest_headers.insert("Referer", HeaderValue::from_static(PINTEREST_BASE_URL));
    reqwest_headers.insert("X-Requested-With", HeaderValue::from_static("XMLHttpRequest"));
    reqwest_headers.insert("Accept", HeaderValue::from_static("application/json"));
    reqwest_headers.insert("Content-Type", HeaderValue::from_static("application/x-www-form-urlencoded; charset=UTF-8"));

    reqwest_headers
}
