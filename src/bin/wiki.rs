use std::time::Duration;
use pinterest_login::login;

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let email = std::env::var("PINTEREST_EMAIL").unwrap();
    let password = std::env::var("PINTEREST_PASSWORD").unwrap();
    let _cookies = login(email.as_str(), password.as_str(), true,
                         Duration::from_secs(2).into(), Duration::from_secs(4).into()).await?;

    println!("{:?}", _cookies);

    Ok(())
}
