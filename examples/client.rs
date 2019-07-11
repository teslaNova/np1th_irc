extern crate np1th_irc;

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

    let client = Client::connect("irc.freenode.org", Port::Secure(7000), myself, None)?;
    client.disconnect();

    Ok(())
}