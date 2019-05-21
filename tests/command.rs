extern crate np1th_irc;

use np1th_irc::command::{client, Command};

use std::{convert::TryInto, error::Error};

fn test_command<C, P>(tests: Vec<&str>, p: P)
                      where
                          P: Fn(Result<C, Box<Error>>),
                          C: Command,
{
    for raw in tests {
        let raw_res = raw.try_into();

        dbg!(&raw_res);
        assert!(raw_res.is_ok());

        let res = C::try_from(raw_res.unwrap());

        dbg!(&res);

        p(res);
    }
}

#[test]
fn test_user_command() {
    let valid_tests = vec!["USER ~sup * * ::@*Fuck Off, loser."];
    let invalid_tests = vec!["USER ~sup * * ::@*Fuck O\rff, loser.", "USER"];

    test_command(valid_tests, |res: Result<client::Command, Box<Error>>| {
        assert!(res.is_ok())
    });
    test_command(invalid_tests, |res: Result<client::Command, Box<Error>>| {
        assert!(res.is_err())
    });
}

#[test]
fn test_nick_command() {
    let valid_tests = vec!["NICK testasd", "NICK :yoo1"];
    let invalid_tests = vec!["NICK ~sup", "NICK toolonguser1234567890"];

    test_command(valid_tests, |res: Result<client::Command, Box<Error>>| {
        assert!(res.is_ok())
    });
    test_command(invalid_tests, |res: Result<client::Command, Box<Error>>| {
        assert!(res.is_err())
    });
}

#[test]
fn test_join_command() {
    let valid_tests = vec![
        "JOIN 0",
        "JOIN #test",
        "JOIN #test testkey",
        "JOIN #test,&foo testkey",
        "JOIN #test,&foo testkey,someotherkey~",
    ];

    let invalid_tests = vec![
        "JOIN 0 a",
        "JOIN",
        "JOIN #test testkey,another",
        "JOIN -test,&foo testkey",
    ];

    test_command(valid_tests, |res: Result<client::Command, Box<Error>>| {
        assert!(res.is_ok())
    });
    test_command(invalid_tests, |res: Result<client::Command, Box<Error>>| {
        assert!(res.is_err())
    });
}

#[test]
fn test_part_command() {
    let valid_tests = vec!["PART #test", "PART #test,&test2", "PART #test :Bye bye.."];

    test_command(valid_tests, |res: Result<client::Command, Box<Error>>| {
        assert!(res.is_ok())
    });
}
