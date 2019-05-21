use std::error::Error;

pub mod error {
    impl_error!(IllegalServerCommandError {});
}

// RFC 2813
#[derive(Debug)]
pub enum Command {}

impl Command {
    pub fn parse(_cmd: &str, _params: Option<&str>) -> Result<Self, Box<Error>> {
        Err(error::IllegalServerCommandError::new())
    }
}
