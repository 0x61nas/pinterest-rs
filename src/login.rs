use std::path::Path;
use reqwest::cookie::Jar;
use url::Url;
use pinterest_login::config_builder::DefaultBrowserConfigBuilder;
use pinterest_login::login_bot::DefaultBrowserLoginBot;
use crate::{Pinterest, PINTEREST_BASE_URL, PinterestError};
use crate::utils::build_request_headers;


impl Pinterest {
    pub async fn login<S: Into<String>, P: AsRef<Path>>(&mut self, username: S, password: S, cred_path: Option<P>) -> crate::Result<()> {
        let cookies_map = pinterest_login::login(&DefaultBrowserLoginBot::new(username.into().as_str(), password.into().as_str()),
                                                 &DefaultBrowserConfigBuilder::default(),
        ).await?;

        // Get the crf token from the cookies
        let Some(crf_token) = cookies_map.get("csrftoken").map(|s| s.to_owned()) else {
            return Err(PinterestError::MissingCrfToken);
        };

        if let Some(cred_path) = cred_path {
            // Save the cookies to the credentials file as a json string
            let cookies_json = serde_json::to_string(&cookies_map).unwrap();
            std::fs::write(cred_path, cookies_json)?;
        }


        // Setup the client with the cookies
        let jar = Jar::default();

        let mut cookies_str = String::new();
        for cookie in cookies_map {
            cookies_str.push_str(&cookie.0);
            cookies_str.push('=');
            cookies_str.push_str(&cookie.1);
            cookies_str.push(';');
        }

        jar.add_cookie_str(&cookies_str, &Url::parse(PINTEREST_BASE_URL).unwrap());

        self.client = reqwest::Client::builder()
            .cookie_provider(jar.into())
            .default_headers(build_request_headers(
                crf_token,
                self.user_agent.clone(),
            ))
            .build()?;

        Ok(())
    }
}
