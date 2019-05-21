extern crate np1th_irc;

use np1th_irc::parsing;

#[test]
pub fn msg_target_text() {
    let tests = vec!["#channel,nickname", "#channel", "nickname"];

    for test in tests {
        let res = parsing::msg_target(test);

        assert!(res.is_ok());
    }
}
