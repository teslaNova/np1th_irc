use crate::{
    stream::ClientStream,
    user::User,
    channel::Channel,
    server::Server,
    origin::Origin,
    command::client::Command::{*, User as _User},
    utils::Defaults,
};

use std::{
    net::ToSocketAddrs
};

pub use crate::stream::Port;

pub mod error {
    impl_error!(MissingParameterError {parameter: String});
    impl_error!(ConnectionError {error: Box<std::error::Error>});

    impl_error!(InvalidPassword {});
    impl_error!(Error {message: String});
}

#[derive(Default)]
pub struct Builder {
    user: Option<User>,
    //password: Option<String>,
    host: String,
    ports: Vec<Port>,
    ports_filter: Vec<Box<Fn(&mut Vec<Port>)>>,
    timeout: Option<std::time::Duration>,
}

impl Builder {
    pub fn user(mut self, user: User) -> Self {
        // Is user origin correct?
        self.user = Some(user);

        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();

        self
    }

    pub fn ports(mut self, ports: Vec<Port>) -> Self {
        self.ports = ports;

        self
    }

    pub fn port(mut self, port: Port) -> Self {
        self.ports.push(port);

        self
    }

    pub fn secure_only(mut self) -> Self {
        self.ports_filter.push(Box::new(move |ports| {
            for i in ports.len()..0 {
                if let Port::Insecure(_) = ports[i] {
                    ports.remove(i);
                }
            }
        }));

        self
    }

    pub fn insecure_only(mut self) -> Self {
        self.ports_filter.push(Box::new(move |ports| {
            for i in ports.len()..0 {
                if let Port::Secure(_) = ports[i] {
                    ports.remove(i);
                }
            }
        }));

        self
    }

    pub fn priortize_secure(mut self) -> Self {
        self.ports_filter.push(Box::new(move |ports| {
            ports.sort_by(|c1, c2| {
                if c1.secure() && c2.insecure() {
                    std::cmp::Ordering::Less
                } else if c1.insecure() && c2.secure() {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
        }));

        self
    }

    pub fn priortize_insecure(mut self) -> Self {
        self.ports_filter.push(Box::new(move |ports| {
            ports.sort_by(|c1, c2| {
                if c1.secure() && c2.insecure() {
                    std::cmp::Ordering::Greater
                } else if c1.insecure() && c2.secure() {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            })
        }));

        self
    }

    pub fn timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);

        self
    }

    pub fn build(mut self) -> Result<Client, Box<std::error::Error>> {
        for filter in self.ports_filter {
            filter(&mut self.ports);
        }

        let user = match self.user {
            Some(user) => user,
            None => return Err(error::MissingParameterError::new(format!("User")))
        };

        let mut last_error = None;

        for port in self.ports {
            match ClientStream::connect(&self.host, port, self.timeout) {
                Ok(stream) => {
                    return Client::initialize(stream, user, None)
                }

                Err(e) => last_error = Some(e)
            }
        }

        Err(last_error.unwrap())
    }
}

pub struct Client {
    stream: ClientStream,
    myself: User,
    server: Server,
}

impl Client {
    pub fn builder() -> Builder {
        Builder::default()
    }
}

impl Client {
    fn initialize(stream: ClientStream, myself: User, password: Option<&str>) -> Result<Self, Box<std::error::Error>> {
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

    pub fn disconnect(self) {
        let _ = self.stream.send(Quit {
            reason: None
        });

        self.stream.close();
    }
}