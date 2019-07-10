use crate::{
    stream::ClientStream,
    user::User,
    channel::Channel,
    server::Server,
    command::client::Command::{*, User as _User},
};

use std::{
    net::ToSocketAddrs
};
use crate::origin::Origin;

pub struct Client {
    stream: ClientStream,
    myself: User,
    server: Server,
}

pub mod error {
    impl_error!(InvalidPassword {});
    impl_error!(Error {message: String});
}

impl Client {
    pub fn connect<A: ToSocketAddrs>(addr: A, myself: User, password: Option<&str>) -> Result<Self, Box<std::error::Error>> {
        let stream = ClientStream::connect(addr)?;
        let mut server_motd = String::new();
        let mut server_origin = None;

        stream
            .send(Nick {
                name: myself.origin().nick().unwrap().to_string()
            })?
            .send(_User {
                name: myself.origin().user().unwrap().to_string(),
                modes: vec![],
                real_name: myself.real_name().to_string(),
            })?;

        if let Some(password) = password {
            stream.send(Pass { password: password.to_string() })?; // TODO: react on response!
        }

        let queued = stream
            .iter()
            .filter(|m| m.is_some())
            .map(|m| m.unwrap());

        for message in queued {
            match message.command() {
                MotdStart { .. } => server_origin = Some(message.origin().clone()),
                MotdBody { text } => {
                    server_motd.push_str(&format!("{}\n", text));
                }
                MotdEnd { .. } => break,
                ErrorMsg { text } => return Err(error::Error::new(text.to_string())),
                _ => (),
            }
        }

        let mut server = Server::new(server_origin.unwrap());
        server.set_motd(Some(server_motd));

        Ok(Client {
            myself,
            stream,
            server,
        })
    }

    pub fn disconnect(mut self) {
        self.stream.send(Quit {
            reason: None
        });

        self.stream.close();
    }
}