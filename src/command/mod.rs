use std::{error::Error, fmt::Debug};

pub mod client;
pub mod server;

pub mod error {
    //impl_error!(UnknownCommandError {cmd});
}

pub const TRAILING_DELIMITER: &'static str = ":";

#[derive(Debug)]
pub struct RawCommand<'a> {
    pub command: &'a str,
    pub parameters: Vec<&'a str>, // TODO: check if len is max up to 15
}

impl<'a> std::convert::TryInto<RawCommand<'a>> for &'a str {
    type Error = Box<Error>;

    fn try_into(self) -> Result<RawCommand<'a>, Self::Error> {
        let parts = self.split(crate::SEPARATOR).collect::<Vec<&'a str>>();

        Ok(RawCommand {
            command: if parts.len() >= 1 {
                parts[0]
            } else {
                return Err(crate::message::error::IllegalMessageFormatError::new());
            },
            parameters: if parts.len() > 1 {
                parts[1..].to_vec()
            } else {
                Vec::new()
            },
        })
    }
}

pub trait Command: Debug + Clone + ToString + crate::message::ToMessage<Self>
    where
        Self: Sized,
{
    fn try_from<'a>(r: RawCommand<'a>) -> Result<Self, Box<Error>>;
}
