use crate::channel::RcChannel;
use crate::{
    channel::WeakChannel,
    mode::{user, Mode},
    origin::Origin,
};

#[derive(Debug)]
pub struct User {
    origin: Origin,
    real_name: String,
    modes: Vec<Mode<user::Mode>>,
    channels: Vec<WeakChannel>,
}

impl User {
    pub fn new(origin: Origin, real_name: &str) -> Self {
        User {
            origin,
            real_name: real_name.to_string(),
            modes: vec![],
            channels: vec![],
        }
    }
}

impl User {
    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn origin_mut(&mut self) -> &mut Origin {
        &mut self.origin
    }

    pub fn real_name(&self) -> &str {
        self.real_name.as_str()
    }

    pub fn set_real_name(&mut self, real_name: &str) {
        self.real_name.clear();
        self.real_name.push_str(real_name);
    }

    pub fn modes(&self) -> &Vec<Mode<user::Mode>> {
        &self.modes
    }
}

impl User {
    pub fn channels(&self) -> Vec<WeakChannel> {
        self.channels.clone()
    }

    pub fn channel(&self, name: &str) -> Option<WeakChannel> {
        for weak in self.channels.clone() {
            if let Some(channel) = weak.upgrade() {
                if channel.borrow().name() == name {
                    return Some(weak);
                }
            }
        }

        None
    }

    pub fn in_channel(&self, name: &str) -> bool {
        self.channel(name).is_some()
    }
}

pub type WeakUser = std::rc::Weak<std::cell::RefCell<User>>;
pub type RcUser = std::rc::Rc<std::cell::RefCell<User>>;
