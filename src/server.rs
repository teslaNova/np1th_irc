use crate::{channel::RcChannel, origin::Origin, user::RcUser, utils::Defaults};

#[derive(Debug)]
pub enum Limit {
    NickName(usize),
}

impl Defaults for Limit {
    fn defaults() -> Vec<Self> {
        vec![Limit::NickName(crate::limits::NICK_NAME)]
    }
}

#[derive(Debug)]
pub enum Charset {
    NickName(String),
}

impl Defaults for Charset {
    fn defaults() -> Vec<Self> {
        vec![Charset::NickName(crate::charset::NICK_NAME_REM.to_string())]
    }
}

#[derive(Debug, Default)]
pub struct Server {
    origin: Origin,
    ports: (Vec<u16>, Vec<u16>),
    // secure, insecure
    password: Option<String>,
    motd: Option<String>,
    users: Vec<RcUser>,
    channels: Vec<RcChannel>,
    // limits: Vec<Limits>
}

impl Server {
    pub fn new(origin: Origin) -> Self {
        Server {
            origin,
            ..Default::default()
        }
    }
}

impl Server {
    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    pub fn secure_ports(&self) -> &Vec<u16> {
        &self.ports.0
    }

    pub fn insecure_ports(&self) -> &Vec<u16> {
        &self.ports.1
    }

    pub fn users(&self) -> &Vec<RcUser> {
        &self.users
    }

    pub fn users_mut(&mut self) -> &mut Vec<RcUser> {
        &mut self.users
    }

    pub fn channels(&self) -> &Vec<RcChannel> {
        &self.channels
    }

    pub fn channels_mut(&mut self) -> &mut Vec<RcChannel> {
        &mut self.channels
    }

    pub fn motd(&self) -> Option<&str> {
        self.motd.as_ref().map(|motd| motd.as_str())
    }

    pub fn set_motd(&mut self, motd: Option<String>) {
        self.motd = motd;
    }
}

impl std::convert::TryFrom<&Origin> for Server {
    type Error = ();

    fn try_from(origin: &Origin) -> Result<Self, Self::Error> {
        if let Origin::Server { .. } = origin {
            Ok(Server {
                origin: origin.clone(),
                ports: (Vec::new(), Vec::new()), // secure, insecure
                password: None,
                motd: None,
                users: Vec::new(),
                channels: Vec::new(),
            })
        } else {
            Err(())
        }
    }
}
