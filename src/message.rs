use std::{convert::TryInto, error::Error};

use crate::{command, origin, END_OF_MESSAGE};

pub mod error {
    impl_error!(IllegalMessageFormatError {});
    impl_error!(MessageTooLongError {});
}

pub trait ToMessage<C>
    where
        C: command::Command,
{
    fn into_message(self) -> Message<C>;
}

#[derive(Debug)]
/// IRC `Message` representation with an `Origin` and a `Command`
pub struct Message<C> {
    origin: origin::Origin,
    command: C,
}

impl<C> Message<C> {
    pub fn origin(&self) -> &origin::Origin {
        &self.origin
    }

    pub fn command(&self) -> &C {
        &self.command
    }
}

/// Parses a message
impl<'a, C> std::convert::TryFrom<&'a str> for Message<C>
    where
        C: command::Command,
{
    type Error = Box<Error>;

    fn try_from(mut line: &'a str) -> Result<Self, Self::Error> {
        if line.len() > crate::limits::LINE {
            return Err(error::IllegalMessageFormatError::new());
        }

        // ignore EOM
        if line.ends_with(crate::END_OF_MESSAGE) {
            line = &line[..line.len() - 2];
        }

        if let Some(origin_end_pos) = line.find(crate::SEPARATOR) {
            let (origin, command) = if !line.starts_with(origin::PREFIX) {
                (origin::Origin::Connection, C::try_from(line.try_into()?)?)
            } else {
                (
                    line[1..origin_end_pos].try_into()?,
                    C::try_from(line[origin_end_pos + 1..].try_into()?)?,
                )
            };

            Ok(Message { origin, command })
        } else {
            Err(error::IllegalMessageFormatError::new())
        }
    }
}

/// Builds the message
impl<C> ToString for Message<C>
    where
        C: command::Command,
{
    fn to_string(&self) -> String {
        let command = self.command.to_string();
        let origin = self.origin.to_string();

        if origin.is_empty() {
            format!("{}{}", command, END_OF_MESSAGE)
        } else {
            format!("{} {}{}", origin, command, END_OF_MESSAGE)
        }
    }
}

/// Builds `Message` directly from `Command` with its origin set to the connection (no prefix)
impl<C> From<C> for Message<C>
    where
        C: command::Command,
{
    fn from(command: C) -> Self {
        Message {
            origin: origin::Origin::default(),
            command,
        }
    }
}

impl<C> ToMessage<C> for Message<C>
    where
        C: command::Command,
{
    fn into_message(self) -> Message<C> {
        self
    }
}
