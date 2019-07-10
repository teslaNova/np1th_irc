use crate::validate;
use std::error::Error;

pub mod error {
    impl_error!(IllegalOriginFormatError {validation: Box<std::error::Error>});
}

pub const PREFIX: char = ':';

pub const IDENT_SEPARATOR: char = '!';
pub const HOST_SEPARATOR: char = '@';

#[derive(Debug, Clone, PartialEq)]
/// Represents the IRC prefix
pub enum Origin {
    Connection,

    User {
        nick: String,
        user: Option<String>,
        host: Option<String>,
    },

    Server {
        name: String,
    },
}

impl Origin {
    fn parse_server(data: &str) -> Result<Self, Box<Error>> {
        crate::validate::host_name(data).and_then(|_| {
            Ok(Origin::Server {
                name: String::from(data),
            })
        })
    }

    fn parse_user(data: &str) -> Result<Self, Box<Error>> {
        let total = data.len();

        let ident_start_pos = data
            .find(|c| c == IDENT_SEPARATOR)
            .and_then(|p| Some(p + 1))
            .unwrap_or(total);

        let host_start_pos = data
            .rfind(|c| c == HOST_SEPARATOR)
            .and_then(|p| Some(p + 1))
            .unwrap_or(total);

        let nick_end_pos = if ident_start_pos == total {
            total
        } else {
            ident_start_pos - 1
        };

        let nick_part = &data[0..nick_end_pos];

        validate::nick_name(nick_part)?;

        let host = if host_start_pos < total {
            let _tmp = &data[host_start_pos..total];

            validate::host_name(_tmp)?;

            Some(String::from(_tmp))
        } else {
            None
        };

        let user = if ident_start_pos < host_start_pos {
            let _tmp = &data[ident_start_pos..host_start_pos - 1];

            validate::user_name(_tmp)?;

            Some(String::from(_tmp))
        } else {
            None
        };

        Ok(Origin::User {
            nick: String::from(nick_part),

            user,
            host,
        })
    }
}

impl Origin {
    pub fn nick(&self) -> Option<&str> {
        if let Origin::User { ref nick, .. } = self {
            Some(&nick)
        } else {
            None
        }
    }

    pub fn user(&self) -> Option<&str> {
        match self {
            Origin::User { ref user, .. } if user.is_some() => user.as_ref().map(|s| s.as_str()),
            _ => None,
        }
    }

    pub fn host(&self) -> Option<&str> {
        match self {
            Origin::User { ref host, .. } if host.is_some() => host.as_ref().map(|s| s.as_str()),
            _ => None,
        }
    }
}

impl Origin {
    pub fn is_user(&self) -> bool {
        match self {
            Origin::User { .. } => true,
            _ => false,
        }
    }

    pub fn is_server(&self) -> bool {
        match self {
            Origin::Server { .. } => true,
            _ => false,
        }
    }

    pub fn is_connection(&self) -> bool {
        match self {
            Origin::Connection => true,
            _ => false,
        }
    }
}

/// Parses prefix
impl std::convert::TryFrom<&str> for Origin {
    type Error = Box<Error>;

    fn try_from(data: &str) -> Result<Self, Self::Error> {
        Origin::parse_user(data)
            .or(Origin::parse_server(data))
            .map_err(|e| error::IllegalOriginFormatError::new(e))
    }
}

impl Default for Origin {
    fn default() -> Self {
        Origin::Connection
    }
}

/// Builds prefix
impl ToString for Origin {
    fn to_string(&self) -> String {
        match self {
            Origin::User {
                ref nick,
                ref user,
                ref host,
            } => {
                let _user = user.as_ref().map(|u| format!("!{}", u)).unwrap_or_default();

                let _host = host.as_ref().map(|h| format!("@{}", h)).unwrap_or_default();

                format!(":{}{}{}", nick, _user, _host)
            }

            Origin::Server { ref name } => format!(":{}", name),

            _ => format!(""),
        }
    }
}
