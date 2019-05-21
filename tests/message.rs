extern crate np1th_irc;

use np1th_irc::{command::client, message::Message};

use std::convert::TryFrom;

#[test]
fn test_nick_client_message() {
    let message_res =
        Message::<client::Command>::try_from(":avona1!~avon1@localhost NICK :whatever\r\n");

    assert!(message_res.is_ok());
}
