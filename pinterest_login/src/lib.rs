use std::collections::HashMap;
use std::time::Duration;

use async_std::prelude::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};

pub const PINTEREST_LOGIN_URL: &str = "https://pinterest.com/login";

pub async fn login(email: &str, password: &str, headless: bool, request_timeout: Option<Duration>, lunch_timeout: Option<Duration>)
                   -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
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


    let (browser, mut handler) = Browser::launch(browser_config_builder.build()?).await?;

    let handle = async_std::task::spawn(async move {
        loop {
            let _event = handler.next().await.unwrap();
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

    let mut cookies = HashMap::new();

    // Wait for the page to load, and then get the cookies
    let c = page.wait_for_navigation().await?
        .get_cookies().await?;

    for cookie in c {
        cookies.insert(cookie.name, cookie.value);
    }

    // Cancel the event handler
    handle.cancel().await;

    Ok(cookies)
}
