use std::error::Error;

use crate::*;

pub mod error {
    impl_error!(IllegalLengthError {
        validate: String,
        min: usize,
        max: usize,
        actual: usize
    });
    impl_error!(IllegalCharacterError {
        validate: String,
        character: char,
        position: usize
    });
}

fn is_valid_first_rest(
    data: &str,
    charset_first: &str,
    charset_rem: &str,
) -> Result<(), Box<Error>> {
    let is_first_valid = charset_first
        .chars()
        .any(|cs| cs == data.chars().nth(0).unwrap());

    if !is_first_valid {
        return Err(error::IllegalCharacterError::new(
            data.to_string(),
            data.chars().nth(0).unwrap(),
            1,
        ));
    }

    let illegal = &data[1..]
        .chars()
        .enumerate()
        .filter(|(_, c)| charset_rem.find(|cs| c == &cs).is_none())
        .collect::<Vec<(usize, char)>>();

    if illegal.is_empty() {
        Ok(())
    } else {
        Err(error::IllegalCharacterError::new(
            data.to_string(),
            illegal[0].1,
            illegal[0].0 + 1,
        ))
    }
}

fn contains_not(data: &str, illegal: &str) -> Result<(), Box<Error>> {
    let illegal = data.chars().position(|c| illegal.contains(|n| c == n));

    if let Some(pos) = illegal {
        Err(error::IllegalCharacterError::new(
            data.to_string(),
            data.chars().nth(pos).unwrap(),
            pos,
        ))
    } else {
        Ok(())
    }
}

pub fn nick_name(data: &str) -> Result<(), Box<Error>> {
    if data.is_empty() || data.len() > limits::NICK_NAME {
        Err(error::IllegalLengthError::new(
            data.to_string(),
            1,
            limits::NICK_NAME,
            data.len(),
        ))
    } else {
        is_valid_first_rest(data, charset::NICK_NAME_FIRST, charset::NICK_NAME_REM)
    }
}

pub fn user_name(data: &str) -> Result<(), Box<Error>> {
    if data.is_empty() {
        Err(error::IllegalLengthError::new(data.to_string(), 1, 100, 0))
    } else {
        contains_not(data, charset::USER_NAME_NOT)
    }
}

pub fn real_name(data: &str) -> Result<(), Box<Error>> {
    contains_not(data, charset::REAL_NAME_NOT)
}

pub fn host_name(data: &str) -> Result<(), Box<Error>> {
    if data.is_empty() || data.len() > limits::HOST_NAME {
        Err(error::IllegalLengthError::new(
            data.to_string(),
            1,
            limits::HOST_NAME,
            data.len(),
        ))
    } else {
        is_valid_first_rest(data, charset::HOST_NAME_FIRST, charset::HOST_NAME_REM)
    }
}

pub fn channel_name(data: &str) -> Result<(), Box<Error>> {
    if data.len() < 2 || data.len() > limits::CHANNEL_NAME {
        Err(error::IllegalLengthError::new(
            data.to_string(),
            2,
            limits::CHANNEL_NAME,
            data.len(),
        ))
    } else if !charset::CHANNEL_PREFIX.contains(data.chars().nth(0).unwrap()) {
        Err(error::IllegalCharacterError::new(
            data.to_string(),
            data.chars().nth(0).unwrap(),
            0,
        ))
    } else {
        contains_not(&data[1..], charset::CHANNEL_NAME_NOT)
    }
}

pub fn key(data: &str) -> Result<(), Box<Error>> {
    if data.is_empty() {
        Err(error::IllegalLengthError::new(
            data.to_string(),
            1,
            1000,
            data.len(),
        ))
    } else {
        contains_not(data, charset::CHANNEL_KEY_NOT)
    }
}
