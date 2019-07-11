extern crate np1th_irc;

use std::time::Duration;

use np1th_irc::{
    connection::client::{Client, Port},
    user::User,
    origin::Origin,
};

fn main() -> Result<(), Box<std::error::Error>> {
    let myself = User::new(Origin::User {
        nick: "avonarret".to_string(),
        user: Some("~avon".to_string()),
        host: None,
    }, "avonarret");

    let client = Client::builder()
        .port(Port::Insecure(6667))
        .port(Port::Secure(7000))
        .host("irc.freenode.org")
        .user(myself)
        .priortize_secure()
        .timeout(Duration::from_secs(5))
        .build()?;

    client.disconnect();

    Ok(())
}