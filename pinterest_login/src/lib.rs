//! Simple crate to login to Pinterest and get the cookies via chromiumoxide to simulate a browser (open a real browser actually)
//!
//! # Examples
//! ```no_run
//! use pinterest_login::config_builder::DefaultBrowserConfigBuilder;
//! use pinterest_login::login;
//! use pinterest_login::login_bot::DefaultBrowserLoginBot;
//!
//! #[async_std::main]
//! async fn main() {
//!     let email = std::env::var("PINTEREST_EMAIL").unwrap();
//!     let password = std::env::var("PINTEREST_PASSWORD").unwrap();
//!
//!     let bot = DefaultBrowserLoginBot::new(email.as_str(), password.as_str());
//!     let config_builder = DefaultBrowserConfigBuilder::default();
//!
//!     match login(&bot, &config_builder).await {
//!         Ok(cookies) => {
//!             // Store the cookies in a file or something, and do whatever you want with them
//!             // I like the cookies bay the way
//!             // ...
//!             println!("{}", cookies.len());
//!             println!("{:?}", cookies);
//!         }
//!         Err(e) => {
//!             // The login was unsuccessful
//!             eprintln!("The login was unsuccessful: {}", e);
//!         }
//!     };
//! }
//! ```
//! ```no_run
//! use pinterest_login::config_builder::DefaultBrowserConfigBuilder;
//! use pinterest_login::login;
//! use pinterest_login::login_bot::DefaultBrowserLoginBot;
//! use std::time::Duration;
//!
//! #[async_std::main]
//! async fn main() {
//!     let email = std::env::var("PINTEREST_EMAIL").unwrap();
//!     let password = std::env::var("PINTEREST_PASSWORD").unwrap();
//!
//!     let bot = DefaultBrowserLoginBot::new(email.as_str(), password.as_str());
//!
//!    // Show the browser, and set the request timeout to 2 seconds
//!     let config_builder = DefaultBrowserConfigBuilder::new(false, Duration::from_secs(2).into(), None);
//!
//!     match login(&bot, &config_builder).await {
//!         Ok(cookies) => {
//!             // ...
//!         }
//!         Err(e) => {
//!             // The login was unsuccessful
//!             eprintln!("The login was unsuccessful: {}", e);
//!         }
//!     };
//! }
//! ```
//!
//! > This project is part of the [pinterest_rs](https://github.com/anas-elgarhy/pinterest-rs) project
pub mod config_builder;
pub mod login_bot;

use std::collections::HashMap;
use async_std::prelude::StreamExt;
use chromiumoxide::Browser;
use crate::config_builder::BrowserConfigBuilder;
use crate::login_bot::BrowserLoginBot;

/// The pinterest login url
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

/// Logs into Pinterest and returns the cookies as a HashMap
///
/// # Arguments
/// * `login_bot` - The login bot to use to fill and submit the login form
/// * `browser_config_builder` - The browser config builder to use to build the browser config
///
/// # Example
/// ```no_run
/// # use std::collections::HashMap;
/// # use pinterest_login::config_builder::DefaultBrowserConfigBuilder;
/// # use pinterest_login::login;
/// # use pinterest_login::login_bot::DefaultBrowserLoginBot;
///
/// async fn login_to_pinterest(email: &str, password: &str) -> pinterest_login::Result<HashMap<String, String>> {
///     let browser_config_builder = DefaultBrowserConfigBuilder::default();
///     let bot = DefaultBrowserLoginBot::new(email, password);
///
///     let cookies = login(&bot, &browser_config_builder).await?;
///     Ok(cookies)
/// }
/// ```
///
/// # Errors
/// * `CdpError` - If there is an error with chromiumoxide (like launching timeout, or request timeout, network error, etc.)  see [chromiumoxide::error::CdpError](https://docs.rs/chromiumoxide/latest/chromiumoxide/error/enum.CdpError.html) to see all the errors
/// * `BrowserConfigBuildError` - If there is an error building the browser config
/// * `AuthenticationError` - If the email or password is incorrect
///
#[inline]
pub async fn login(login_bot: &dyn BrowserLoginBot, config_builder: &dyn BrowserConfigBuilder)
                   -> Result<HashMap<String, String>> {
    let (browser, mut handler) = Browser::launch(config_builder.build_browser_config()?).await?;

    let handle = async_std::task::spawn(async move {
        loop {
            let _event = handler.next().await;
        }
    });

    let page = browser.new_page(PINTEREST_LOGIN_URL).await?;

    // Fill the login form
    login_bot.fill_login_form(&page).await?;
    // Click the login button
    login_bot.submit_login_form(&page).await?;
    // Check if the login was successful
    login_bot.check_login(&page).await?;


    let mut cookies = HashMap::with_capacity(5);

    // Get the cookies
    let c = page.get_cookies().await?;

    for cookie in c {
        cookies.insert(cookie.name, cookie.value);
    }

    // Cancel the event handler
    handle.cancel().await;

    Ok(cookies)
}
