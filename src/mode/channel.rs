use std::error::Error;

use crate::mode;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Limit { value: usize },
}

impl mode::Parseable for Mode {
    type Target = Self;

    fn parse(data: &str) -> Result<Self::Target, Box<Error>> {
        if data.len() != 1 {
            Err(mode::error::IllegalModeError::new())
        } else {
            match data.chars().nth(0).unwrap() {
                _ => Err(mode::error::IllegalModeError::new()),
            }
        }
    }
}
