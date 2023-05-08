use std::path::Path;
use std::time::Duration;
use reqwest::cookie::Jar;
use url::Url;
use pinterest_login::config_builder::{BrowserConfigBuilder, DefaultBrowserConfigBuilder};
use pinterest_login::login_bot::{BrowserLoginBot, DefaultBrowserLoginBot};
#[cfg(feature = "debug")]
use log::*;

use crate::{Pinterest, PINTEREST_BASE_URL, PinterestError};
use crate::utils::build_request_headers;

impl Pinterest {
    #[inline]
    pub async fn login<S: Into<String>, P: AsRef<Path>>(&mut self, email: S, password: S, cred_path: Option<P>) -> crate::Result<()> {
        self.login_with_custom_bot_and_config(&DefaultBrowserLoginBot::new(email.into().as_str(), password.into().as_str()),
                                              &DefaultBrowserConfigBuilder::new(true, Some(Duration::from_secs(8)), None),
                                              cred_path).await
    }

    #[inline(always)]
    pub async fn login_with_custom_bot_and_config<P: AsRef<Path>>(&mut self, bot: &impl BrowserLoginBot,
                                                                  config: &impl BrowserConfigBuilder,
                                                                  cred_path: Option<P>) -> crate::Result<()> {
        #[cfg(feature = "debug")] info!("Logging in with custom bot and config");

        let cookies_map = pinterest_login::login(bot, config).await?;

        #[cfg(feature = "debug")] {
            info!("Successfully logged in");
            trace!("Cookies: {:?}", cookies_map);
            debug!("Cookies count: {}", cookies_map.len());
            debug!("Keys: {:?}", cookies_map.keys());
        }

        // Get the crf token from the cookies
        let Some(crf_token) = cookies_map.get("csrftoken").map(|s| s.to_owned()) else {
            #[cfg(feature = "debug")] error!("The crf token was not found in the cookies");
            return Err(PinterestError::MissingCrfToken);
        };

        #[cfg(feature = "debug")] {
            trace!("Crf token: {}", crf_token);
            debug!("Crf token length: {}", crf_token.len());
            debug!("Credentials path is set: {}", cred_path.is_some());
        }

        if let Some(cred_path) = cred_path {
            #[cfg(feature = "debug")] {
                info!("Saving the cookies to the credentials file");
                trace!("Credentials path: {:?}", cred_path.as_ref());
            }
            // Save the cookies to the credentials file as a json string
            let cookies_json = serde_json::to_string(&cookies_map).unwrap();
            #[cfg(feature = "debug")] trace!("Cookies json: {}", cookies_json);
            std::fs::write(cred_path, cookies_json)?;
            #[cfg(feature = "debug")] info!("Successfully saved the cookies to the credentials file");
        }

        #[cfg(feature = "debug")] info!("Setting up the client with the cookies");

        // Setup the client with the cookies
        let jar = Jar::default();

        let mut cookies_str = String::new();
        for cookie in cookies_map {
            cookies_str.push_str(&cookie.0);
            cookies_str.push('=');
            cookies_str.push_str(&cookie.1);
            cookies_str.push(';');
        }

        #[cfg(feature = "debug")] {
            debug!("Cookies string length: {}", cookies_str.len());
            trace!("Cookies string: {}", cookies_str);
            debug!("Adding the cookies to the cookie jar with the base url: {}", PINTEREST_BASE_URL);
        }

        jar.add_cookie_str(&cookies_str, &Url::parse(PINTEREST_BASE_URL).unwrap());

        #[cfg(feature = "debug")] {
            debug!("Building the client with the cookie jar and the request headers");
            trace!("User agent: {:?}", self.user_agent);
        }

        self.client = reqwest::Client::builder()
            .cookie_provider(jar.into())
            .default_headers(build_request_headers(
                crf_token,
                self.user_agent.clone(),
            ))
            .build()?;

        Ok(())
    }

    #[inline]
    pub async fn login_with_timeout<S: Into<String>, P: AsRef<Path>>(&mut self, email: S, password: S,
                                                                     request_timeout: Duration,
                                                                     browser_launch_timeout: Duration,
                                                                     cred_path: Option<P>) -> crate::Result<()> {
        #[cfg(feature = "debug")] {
            trace!("Request timeout: {:?}", request_timeout);
            trace!("Browser launch timeout: {:?}", browser_launch_timeout);
        }

        self.login_with_custom_bot_and_config(&DefaultBrowserLoginBot::new(email.into().as_str(), password.into().as_str()),
                                              &DefaultBrowserConfigBuilder::new(true, Some(request_timeout), Some(browser_launch_timeout)),
                                              cred_path).await
    }
}
