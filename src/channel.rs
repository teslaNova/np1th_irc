use crate::{
    mode::{channel, Mode},
    user::WeakUser,
};

#[derive(Debug)]
pub struct Channel {
    name: String,
    topic: Option<String>,
    modes: Vec<Mode<channel::Mode>>,
    users: Vec<WeakUser>,
    //password: Option<String>,
}

impl Channel {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn topic(&self) -> Option<&str> {
        self.topic.as_ref().map(|s| s.as_str())
    }

    pub fn modes(&self) -> &Vec<Mode<channel::Mode>> {
        &self.modes
    }

    //pub fn password(&self) ->
}

impl Channel {
    pub fn users(&self) -> Vec<WeakUser> {
        self.users.clone()
    }
}

pub type WeakChannel = std::rc::Weak<std::cell::RefCell<Channel>>;
pub type RcChannel = std::rc::Rc<std::cell::RefCell<Channel>>;
