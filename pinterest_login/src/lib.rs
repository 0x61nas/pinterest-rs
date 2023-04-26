use std::collections::HashMap;
use std::time::Duration;

use async_std::prelude::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};

pub const PINTEREST_LOGIN_URL: &str = "https://pinterest.com/login";

#[derive(Debug, thiserror::Error)]
pub enum PinterestLoginError {
    #[error("{0}")]
    CdpError(#[from] chromiumoxide::error::CdpError),
    #[error("{0}")]
    BrowserConfigBuildError(String),
    #[error("Authentication error: The email or password you entered is incorrect.")]
    AuthenticationError,
}

/// A type alias for `Result<T, PinterestLoginError>`
pub type Result<T> = std::result::Result<T, PinterestLoginError>;

pub async fn login(email: &str, password: &str, headless: bool, request_timeout: Option<Duration>, lunch_timeout: Option<Duration>)
                   -> Result<HashMap<String, String>> {
    let (browser, mut handler) = Browser::launch(
        build_browser_config(headless, request_timeout, lunch_timeout)?).await?;

    let handle = async_std::task::spawn(async move {
        loop {
            let _event = handler.next().await;
        }
    });

    let page = browser.new_page(PINTEREST_LOGIN_URL).await?;

    // Wait for the page to load, and then find the email input field and fill it
    page.wait_for_navigation().await?
        .find_element("input#email").await?
        .click().await?
        .type_str(email).await?;
    // Find the password input field and fill it
    page.find_element("input#password").await?
        .click().await?
        .type_str(password).await?;

    // Find the submit button and click it
    page.find_element("button[type='submit']").await?
        .click().await?;

    // Wait for the page to load, and then check if the login was successful
    match page.wait_for_navigation().await?.url().await? {
        None => {
            // If we can't get the url, then the login was unsuccessful
            return Err(PinterestLoginError::AuthenticationError);
        }
        Some(url) => {
            if url == PINTEREST_LOGIN_URL {
                // If the url is the same as the login url, then the login was unsuccessful
                return Err(PinterestLoginError::AuthenticationError);
            }
        }
    }


    let mut cookies = HashMap::new();

    // Get the cookies
    let c = page.get_cookies().await?;

    for cookie in c {
        cookies.insert(cookie.name, cookie.value);
    }

    // Cancel the event handler
    handle.cancel().await;

    Ok(cookies)
}

#[inline(always)]
fn build_browser_config(headless: bool, request_timeout: Option<Duration>, lunch_timeout: Option<Duration>)
                        -> Result<BrowserConfig> {
    let mut browser_config_builder = if headless {
        BrowserConfig::builder()
    } else {
        BrowserConfig::builder().with_head()
    };

    if let Some(timeout) = request_timeout {
        browser_config_builder = browser_config_builder.request_timeout(timeout);
    }

    if let Some(timeout) = lunch_timeout {
        browser_config_builder = browser_config_builder.launch_timeout(timeout);
    }

    browser_config_builder.build().map_err(PinterestLoginError::BrowserConfigBuildError)
}
