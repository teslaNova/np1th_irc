use std::error::Error;

use crate::{command::TRAILING_DELIMITER, validate, LIST_ITEM_DELIMITER, SEPARATOR};

fn current_parameter(data: &str) -> &str {
    if let Some(pos) = data.find(SEPARATOR) {
        &data[..pos]
    } else {
        data
    }
}

pub fn to_parameter_list(data: &str) -> Vec<&str> {
    data.split(SEPARATOR).collect::<Vec<&str>>()
}

pub fn skip_parameter(data: &str) -> &str {
    let pos = data
        .find(SEPARATOR)
        .map(|pos| pos + 1)
        .unwrap_or(data.len());

    &data[pos..]
}

pub fn skip_maybe_trailing(data: &str) -> &str {
    if &data[..1] == TRAILING_DELIMITER {
        &data[1..]
    } else {
        data
    }
}

fn list(data: &str) -> Vec<&str> {
    data.split(LIST_ITEM_DELIMITER).collect::<Vec<&str>>()
}

pub fn nick_name(data: &str) -> Result<&str, Box<Error>> {
    let tmp = current_parameter(data);

    validate::nick_name(tmp).and_then(|_| Ok(tmp))
}

pub fn user_name(data: &str) -> Result<&str, Box<Error>> {
    let tmp = current_parameter(data);

    validate::user_name(tmp).and_then(|_| Ok(tmp))
}

pub fn real_name(data: &str) -> Result<&str, Box<Error>> {
    validate::real_name(data).and_then(|_| Ok(data))
}

pub fn key(data: &str) -> Result<&str, Box<Error>> {
    let tmp = current_parameter(data);

    validate::key(tmp).and_then(|_| Ok(tmp))
}

pub fn channel_name(data: &str) -> Result<&str, Box<Error>> {
    let tmp = current_parameter(data);

    validate::channel_name(tmp).and_then(|_| Ok(tmp))
}

pub fn server_name(data: &str) -> Result<&str, Box<Error>> {
    let tmp = current_parameter(data);

    validate::host_name(tmp).and_then(|_| Ok(tmp))
}

pub fn target(data: &str) -> Result<&str, Box<Error>> {
    (nick_name(data).or(server_name(data))).and_then(|_| Ok(data))
}

pub fn msg_target(data: &str) -> Result<Vec<&str>, Box<Error>> {
    let _targets = current_parameter(data);
    let targets = list(_targets);

    let targets_iter = targets.iter().map(|target| msg_to(target));

    let mut errors = targets_iter
        .clone()
        .filter(|r| r.is_err())
        .map(|r| r.err().unwrap())
        .collect::<Vec<Box<Error>>>();

    if errors.is_empty() {
        Ok(targets_iter.map(|t| t.unwrap()).collect())
    } else {
        Err(errors.pop().unwrap())
    }
}

pub fn msg_to(data: &str) -> Result<&str, Box<Error>> {
    // TODO: 2 out of 6 implemented..
    channel_name(data).or(nick_name(data))
}
