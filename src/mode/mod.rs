use std::{convert::TryFrom, error::Error};

pub mod channel;
pub mod user;

pub mod error {
    impl_error!(IllegalModeError {});
}

pub trait Parseable: std::fmt::Debug + Clone + PartialEq<Self> {
    type Target: Parseable;

    fn parse(data: &str) -> Result<Self::Target, Box<Error>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mode<T: Parseable> {
    granted: bool,
    mode: T::Target,
}

impl<T: Parseable> Mode<T> {
    pub fn parse(data: &str) -> Result<Self, Box<Error>> {
        let granted = match &data[..1] {
            "+" => true,
            "-" => false,
            _ => return Err(error::IllegalModeError::new()),
        };

        let mode = T::parse(&data[1..])?;

        Ok(Mode { granted, mode })
    }
}

impl<T: Parseable> TryFrom<char> for Mode<T> {
    type Error = Box<Error>;

    fn try_from(mode: char) -> Result<Self, Self::Error> {
        Ok(Self {
            granted: true,
            mode: T::parse(&format!("{}", mode))?,
        })
    }
}
