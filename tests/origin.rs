extern crate np1th_irc;

use std::convert::TryFrom;

use np1th_irc::origin::Origin;

#[test]
fn parse_valid_server_origin() {
    let tests = vec![":127.0.0.1", ":localhost", ":something-interesting.de"];

    for test in tests {
        let res = Origin::try_from(test);

        dbg!(&res);

        assert!(res.is_ok());
    }
}

#[test]
fn parse_invalid_server_origin() {
    use np1th_irc::origin::Origin;

    let tests = vec![
        ":.127.0.0.1",
        ":-localhost",
        ":\rsomething-interesting.de",
        ":thisissomecrazyasslonghostnameandthisisgonnafailorsomethingdoesnotworkasitshouldandthatisbadmkaysohopefullythiswillfailasitshouldasisaidbefore.xyz"
    ];

    for test in tests {
        let res = Origin::try_from(test);

        dbg!(&res);

        assert!(res.is_err());
    }
}

#[test]
fn parse_valid_user_origin() {
    use np1th_irc::origin::Origin;

    let tests = vec![
        ":nick!user@host",
        ":nicki!!user@host",
        ":foo012!baz@127.0.0.1-lol.just.kidding.de",
    ];

    for test in tests {
        let res = Origin::try_from(test);

        dbg!(&test);

        assert!(res.is_ok());
    }
}

#[test]
fn parse_invalid_user_origin() {
    use np1th_irc::origin::Origin;

    let tests = vec![
        ":nicktoolong123456789!user@host",
        ":nicki!!user@@host",
        ":foo012!baz@-127.0.0.1-lol.just.kidding.de",
    ];

    for test in tests {
        let res = Origin::try_from(test);

        dbg!(&test);

        assert!(res.is_err());
    }
}
