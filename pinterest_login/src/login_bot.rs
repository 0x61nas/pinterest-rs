use chromiumoxide::Page;

use crate::{PINTEREST_LOGIN_URL, PinterestLoginError};

/// Trait for login bots, which are used to fill and submit the login form in the browser
///
/// # Example
/// ```no_run
/// use chromiumoxide::Page;
/// # use pinterest_login::login_bot::BrowserLoginBot;
///
/// struct MyLoginBot;
///
/// #[async_trait::async_trait]
/// impl BrowserLoginBot for MyLoginBot {
///    async fn fill_login_form(&self, page: &Page) -> crate::Result<()> {
///        // ...
///    }
///
///    async fn submit_login_form(&self, page: &Page) -> crate::Result<()> {
///        // ...
///    }
///
///    async fn check_login(&self, page: &Page) -> crate::Result<()> {
///        // ...
///    }
/// }
/// ```
#[async_trait::async_trait]
pub trait BrowserLoginBot {
    /// Fills the login form fields with the required data
    async fn fill_login_form(&self, page: &Page) -> crate::Result<()>;
    /// Submits the login form
    async fn submit_login_form(&self, page: &Page) -> crate::Result<()>;
    /// Checks if the login was successful
    async fn check_login(&self, page: &Page) -> crate::Result<()>;
}

/// The default login bot, that provides methods to fill and submit the login form in the browser
/// This login bot enables you to login to pinterest with an email and password
///
/// # Example
/// ```no_run
/// # use pinterest_login::login_bot::{BrowserLoginBot, DefaultBrowserLoginBot};
///
/// let login_bot = DefaultBrowserLoginBot::new("email", "password");
///
/// // ...
/// ```
///
/// U don't need to use the login bot directly, it is used by the login function,
/// you just send it to the login function and it will use it to fill and submit the login form
pub struct DefaultBrowserLoginBot<'a> {
    email: &'a str,
    password: &'a str,
}

impl<'a> DefaultBrowserLoginBot<'a> {
    /// Creates a new default login bot
    ///
    /// # Arguments
    /// * `email` - The email to login with
    /// * `password` - The password to login with
    pub fn new(email: &'a str, password: &'a str) -> Self {
        Self {
            email,
            password,
        }
    }
}

#[async_trait::async_trait]
impl BrowserLoginBot for DefaultBrowserLoginBot<'_> {
    #[inline(always)]
    async fn fill_login_form(&self, page: &Page) -> crate::Result<()> {
        // Wait for the page to load, and then find the email input field and fill it
        page.wait_for_navigation().await?
            .find_element("input#email").await?
            .click().await?
            .type_str(self.email).await?;
        // Find the password input field and fill it
        page.find_element("input#password").await?
            .click().await?
            .type_str(self.password).await?;

        Ok(())
    }

    #[inline(always)]
    async fn submit_login_form(&self, page: &Page) -> crate::Result<()> {
        // Find the submit button and click it
        page.find_element("button[type='submit']").await?
            .click().await?;
        Ok(())
    }

    #[inline(always)]
    async fn check_login(&self, page: &Page) -> crate::Result<()> {
        // Wait for the page to load, and then check if the login was successful
        match page.wait_for_navigation().await?.url().await? {
            None => {
                // If we can't get the url, then the login was unsuccessful
                Err(PinterestLoginError::AuthenticationError)
            }
            Some(url) => {
                if url == PINTEREST_LOGIN_URL {
                    // If the url is the same as the login url, then the login was unsuccessful
                    Err(PinterestLoginError::AuthenticationError)
                } else {
                    Ok(())
                }
            }
        }
    }
}
