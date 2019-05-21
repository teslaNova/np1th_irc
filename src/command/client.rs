use std::error::Error;

use crate::{
    command::{RawCommand, TRAILING_DELIMITER},
    message::{Message, ToMessage},
    mode::{channel, user, Mode},
    parsing, validate, SEPARATOR,
};

pub mod error {
    impl_error!(ClientCommandNotImplementedError { cmd: String });
    impl_error!(IllegalClientCommandError { cmd: String });
    impl_error!(ClientCommandParameterError {
        cmd: String,
        params: String
    });
}

// RFC 2812
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    // Connection
    Nick {
        name: String,
    },
    User {
        name: String,
        modes: Vec<Mode<user::Mode>>,
        real_name: String,
    },
    Pass {
        password: String,
    },
    Oper {
        name: String,
        password: String,
    },
    UMode {
        name: String,
        modes: Vec<Mode<user::Mode>>,
    },
    Service {
        name: String,
        server_mask: String,
        info: String,
    },
    Quit {
        reason: Option<String>,
    },
    SQuit {
        name: String,
        reason: String,
    },

    // Channel
    Join {
        channels: Vec<String>,
        keys: Vec<String>,
    },
    Join0 {},
    // Who came up with that shit?
    Part {
        channels: Vec<String>,
        reason: Option<String>,
    },
    CMode {
        channel: String,
        modes: Vec<Mode<channel::Mode>>,
    },
    Topic {
        channel: String,
        text: Option<String>,
    },
    Names {
        channels: Option<Vec<String>>,
        server: Option<String>,
    },
    List {
        channels: Option<Vec<String>>,
        server: Option<String>,
    },
    Invite {
        user: String,
        channel: String,
    },
    Kick {
        channels: Vec<String>,
        users: Vec<String>,
        reason: Option<String>,
    },

    // Messages
    PrivMsg {
        targets: Vec<String>,
        text: String,
    },
    Notice {
        target: String,
        text: String,
    },

    // Server
    Motd {
        server: Option<String>,
    },
    LUsers {
        mask: Option<String>,
        server: Option<String>,
    },
    Version {
        server: Option<String>,
    },
    Stats {
        query: Option<String>,
        server: Option<String>,
    },
    Links {
        server: Option<String>,
        mask: Option<String>,
    },
    // none | mask | server, mask (in order)
    Time {
        server: Option<String>,
    },
    Connect {
        server: String,
        port: u16,
        remote: Option<String>,
    },
    Trace {
        server: Option<String>,
    },
    Admin {
        server: Option<String>,
    },
    Info {
        server: Option<String>,
    },

    // Service
    ServList {
        mask: Option<String>,
        kind: String,
    },
    SQuery {
        name: String,
        text: String,
    },

    // User
    Who {
        mask: String,
        operators_only: bool,
    },
    WhoIs {
        server: Option<String>,
        masks: Vec<String>,
    },
    WhoWas {
        users: Vec<String>,
        count: Option<i32>,
        server: Option<String>,
    },

    // Misc
    Kill {
        user: String,
        reason: String,
    },
    Ping {
        server1: String,
        server2: Option<String>,
    },
    Pong {
        server1: String,
        server2: Option<String>,
    },
    ErrorMsg {
        text: String,
    },
    // Server
    // - Replies
    /* 375 */ MotdStart,
    /* 372 */ MotdBody {
        text: String,
    },
    /* 376 */ MotdEnd, // - Errors
}

impl crate::command::Command for Command {
    fn try_from<'a>(r: RawCommand<'a>) -> Result<Self, Box<Error>> {
        let not_implemented_err: Result<Self, Box<Error>> = Err(
            error::ClientCommandNotImplementedError::new(r.command.to_string()),
        );

        match r.command {
            // Connection
            "NICK" => {
                if r.parameters.len() == 1 {
                    if let Ok(nick) =
                    parsing::nick_name(parsing::skip_maybe_trailing(r.parameters[0]))
                    {
                        return Ok(Command::Nick {
                            name: nick.to_string(),
                        });
                    }
                }
            }

            "USER" => {
                if r.parameters.len() >= 4 && r.parameters[3].starts_with(TRAILING_DELIMITER) {
                    let user_name_res = parsing::user_name(r.parameters[0]);
                    let real_name_joined = &r.parameters[3..].join(SEPARATOR)[1..];
                    let real_name_res = parsing::real_name(real_name_joined);

                    if let (Ok(nick_name), Ok(real_name)) = (user_name_res, real_name_res) {
                        return Ok(Command::User {
                            name: nick_name.to_string(),
                            real_name: real_name.to_string(),
                            modes: Vec::new(),
                        });
                    }
                }
            }

            "PASS" => return not_implemented_err,
            "OPER" => return not_implemented_err,
            "SERVICE" => return not_implemented_err,
            "QUIT" => return not_implemented_err,
            "SQUIT" => return not_implemented_err,

            // Channel
            "JOIN" => {
                if r.parameters.len() == 1 && &r.parameters[0] == &"0" {
                    return Ok(Command::Join0 {}.into());
                } else {
                    if r.parameters.len() >= 1 && r.parameters.len() <= 2 {
                        let mut key_parts_count = 0usize;
                        let channel_parts = r.parameters[0].split(',').collect::<Vec<&str>>();
                        let key_parts = if r.parameters.len() == 2 {
                            r.parameters[1].split(',').collect::<Vec<&str>>()
                        } else {
                            Vec::<&str>::new()
                        };

                        let channels_valid = channel_parts
                            .iter()
                            .filter(|channel| validate::channel_name(channel).is_ok())
                            .count()
                            == channel_parts.len();

                        let keys_valid = key_parts
                            .iter()
                            .filter(|key| validate::key(key).is_ok())
                            .count()
                            == key_parts.len();

                        if channels_valid && keys_valid {
                            let channels = channel_parts
                                .iter()
                                .map(|name| format!("{}", name))
                                .collect::<Vec<String>>();

                            let keys = key_parts
                                .iter()
                                .map(|name| format!("{}", name))
                                .collect::<Vec<String>>();

                            return Ok(Command::Join { channels, keys });
                        }
                    }
                }
            }

            "PART" => {
                if r.parameters.len() == 1
                    || (r.parameters.len() >= 2
                    && r.parameters[1].starts_with(crate::command::TRAILING_DELIMITER))
                {
                    let channel_parts = r.parameters[0].split(',').collect::<Vec<&str>>();
                    let reason = if r.parameters.len() >= 2 && r.parameters[1].len() >= 2 {
                        Some(format!(
                            "{}",
                            &r.parameters[1..].join(crate::SEPARATOR)[1..]
                        ))
                    } else {
                        None
                    };

                    let channels_valid = channel_parts
                        .iter()
                        .filter(|channel| validate::channel_name(channel).is_ok())
                        .count()
                        == channel_parts.len();

                    if channels_valid {
                        let channels = channel_parts
                            .iter()
                            .map(|name| format!("{}", name))
                            .collect::<Vec<String>>();

                        return Ok(Command::Part { channels, reason });
                    }
                }
            }

            "TOPIC" => return not_implemented_err,
            "NAMES" => return not_implemented_err,
            "LIST" => return not_implemented_err,
            "INVITE" => return not_implemented_err,
            "KICK" => return not_implemented_err,

            // Messages
            "PRIVMSG" => {
                /*
                  msgtarget  =  msgto *( "," msgto )
                  msgto      =  channel / ( user [ "%" host ] "@" servername )
                  msgto      =/ ( user "%" host ) / targetmask
                  msgto      =/ nickname / ( nickname "!" user "@" host )
                */

                if r.parameters.len() >= 2 && &r.parameters[1][..1] == TRAILING_DELIMITER {
                    let target_parts = r.parameters[0]
                        .split(crate::LIST_ITEM_DELIMITER)
                        .collect::<Vec<&str>>();
                    let text = format!(
                        "{}",
                        parsing::skip_maybe_trailing(&r.parameters[1..].join(crate::SEPARATOR))
                    );

                    let targets = target_parts
                        .iter()
                        .filter(|part| {
                            validate::channel_name(part)
                                .or(validate::nick_name(part))
                                .is_ok()
                        })
                        .map(|part| format!("{}", part))
                        .collect::<Vec<String>>();

                    return Ok(Command::PrivMsg { targets, text });
                }
            }
            "NOTICE" => {
                if r.parameters.len() >= 2 {
                    return Ok(Command::Notice {
                        target: r.parameters[0].to_string(),
                        text: parsing::skip_maybe_trailing(
                            &r.parameters[1..].join(crate::SEPARATOR),
                        )
                            .to_string(),
                    }
                        .into());
                }
            }

            // Server
            "MOTD" => return not_implemented_err,
            "LUSERS" => return not_implemented_err,
            "VERSION" => return not_implemented_err,
            "STATS" => return not_implemented_err,
            "LINKS" => return not_implemented_err,
            "TIME" => return not_implemented_err,
            "CONNECT" => return not_implemented_err,
            "TRACE" => return not_implemented_err,
            "ADMIN" => return not_implemented_err,
            "INFO" => return not_implemented_err,

            // Service
            "SERVLIST" => return not_implemented_err,
            "SQUERY" => return not_implemented_err,

            // User
            "WHO" => return not_implemented_err,
            "WHOIS" => return not_implemented_err,
            "WHOWAS" => return not_implemented_err,

            // Misc
            "KILL" => return not_implemented_err,
            "PING" => {
                if r.parameters.len() >= 1 {
                    return Ok(Command::Ping {
                        server1: parsing::skip_maybe_trailing(r.parameters[0]).to_string(),
                        server2: r
                            .parameters
                            .get(1)
                            .map(|s| parsing::skip_maybe_trailing(s).to_string()),
                    });
                }
            }
            "PONG" => {}
            "ERROR" => {
                if r.parameters.len() >= 1 {
                    return Ok(Command::ErrorMsg {
                        text: parsing::skip_maybe_trailing(
                            &r.parameters[..].join(crate::SEPARATOR),
                        )
                            .to_string(),
                    });
                }
            }

            // Replies
            "375" => return Ok(Command::MotdStart.into()),
            "372" => {
                if r.parameters.len() >= 3 && r.parameters[1] == ":-" {
                    return Ok(Command::MotdBody {
                        text: r.parameters[2..].join(crate::SEPARATOR),
                    });
                }
            }
            "376" => return Ok(Command::MotdEnd.into()),

            _ => return Err(error::IllegalClientCommandError::new(r.command.to_string())),
        };

        Err(error::ClientCommandParameterError::new(
            r.command.to_string(),
            r.parameters.join(SEPARATOR),
        ))
    }
}

impl ToString for Command {
    fn to_string(&self) -> String {
        use Command::*;

        match self {
            // Connection
            &Nick { ref name } => format!("NICK :{}", name),
            &User {
                ref name,
                ref real_name,
                ..
            } => format!("USER {} * * :{}", name, real_name),
            &Pass { ref password } => format!("PASS {}", password),
            &Oper {
                ref name,
                ref password,
            } => format!("OPER {} {}", name, password),
            &UMode {
                ref name,
                ref modes,
            } => {
                //format!("MODE {} {}", name, modes.to_string())
                format!("")
            }
            &Service {
                ref name,
                ref server_mask,
                ref info,
            } => format!("SERVICE {} {} :{}", name, server_mask, info),
            &Quit { ref reason } => format!(
                "QUIT{}",
                reason
                    .as_ref()
                    .map(|r| format!(" :{}", r))
                    .unwrap_or_default()
            ),
            &SQuit {
                ref name,
                ref reason,
            } => format!("SQUIT {} :{}", name, reason),

            // Channel
            &Join {
                ref channels,
                ref keys,
            } => format!(
                "JOIN {}{}",
                channels.join(","),
                Some(keys.join(","))
                    .map(|kl| format!(" {}", kl))
                    .unwrap_or_default()
            ),

            &Join0 {} => format!("JOIN 0"),
            &Part {
                ref channels,
                ref reason,
            } => format!(
                "PART {}{}",
                channels.join(","),
                reason
                    .as_ref()
                    .map(|r| format!(" :{}", r))
                    .unwrap_or_default()
            ),
            &CMode {
                ref channel,
                ref modes,
            } => format!(""),
            &Topic {
                ref channel,
                ref text,
            } => format!(
                "TOPIC {}{}",
                channel,
                text.as_ref()
                    .map(|t| format!(" :{}", t))
                    .unwrap_or_default()
            ),
            &Names {
                ref channels,
                ref server,
            } => format!(
                "{}{}",
                channels.as_ref().map(|cl| cl.join(",")).unwrap_or_default(),
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &List {
                ref channels,
                ref server,
            } => format!(
                "{}{}",
                channels.as_ref().map(|cl| cl.join(",")).unwrap_or_default(),
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Invite {
                ref user,
                ref channel,
            } => format!("INVITE {} {}", user, channel),
            &Kick {
                ref channels,
                ref users,
                ref reason,
            } => format!(
                "{} {}{}",
                channels.join(","),
                users.join(","),
                reason
                    .as_ref()
                    .map(|r| format!(" :{}", r))
                    .unwrap_or_default()
            ),

            // Messages
            &PrivMsg {
                ref targets,
                ref text,
            } => format!("PRIVMSG {} :{}", targets.join(","), text),
            &Notice {
                ref target,
                ref text,
            } => format!("NOTICE {} :{}", target, text),

            // Server
            &Motd { ref server } => format!(
                "MOTD{}",
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &LUsers {
                ref mask,
                ref server,
            } => format!(
                "LUSERS{}{}",
                mask.as_ref().map(|m| format!(" {}", m)).unwrap_or_default(),
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Version { ref server } => format!(
                "VERSION{}",
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Stats {
                ref query,
                ref server,
            } => format!(
                "STATS{}{}",
                query
                    .as_ref()
                    .map(|q| format!(" {}", q))
                    .unwrap_or_default(),
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Links {
                ref server,
                ref mask,
            } => format!(""), // none | mask | server, mask (in order)
            &Time { ref server } => format!(
                "TIME{}",
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Connect {
                ref server,
                ref port,
                ref remote,
            } => format!(
                "CONNECT {} {}{}",
                server,
                port,
                remote
                    .as_ref()
                    .map(|r| format!(" {}", r))
                    .unwrap_or_default()
            ),
            &Trace { ref server } => format!(
                "TRACE{}",
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Admin { ref server } => format!(
                "ADMIN{}",
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Info { ref server } => format!(
                "INFO{}",
                server
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),

            // Service
            &ServList { ref mask, ref kind } => format!(""),
            &SQuery { ref name, ref text } => format!("SQUERY {} {}", name, text),

            // User
            &Who {
                ref mask,
                ref operators_only,
            } => format!(""),
            &WhoIs {
                ref server,
                ref masks,
            } => format!(""),
            &WhoWas {
                ref users,
                ref count,
                ref server,
            } => format!(""),

            // Misc
            &Kill {
                ref user,
                ref reason,
            } => format!("KILL {} :{}", user, reason),
            &Ping {
                ref server1,
                ref server2,
            } => format!(
                "PING {}{}",
                server1,
                server2
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &Pong {
                ref server1,
                ref server2,
            } => format!(
                "PONG {}{}",
                server1,
                server2
                    .as_ref()
                    .map(|s| format!(" {}", s))
                    .unwrap_or_default()
            ),
            &ErrorMsg { ref text } => format!("ERROR :{}", text),

            _ => format!(""),
        }
    }
}

impl ToMessage<Command> for Command {
    fn into_message(self) -> Message<Command> {
        Message::from(self)
    }
}
