use std::{
    convert::TryFrom,
    error::Error,
    io::{prelude::*, ErrorKind::WouldBlock},
    net::{Shutdown, TcpStream, ToSocketAddrs},
    time::Duration,
    cell::RefCell,
};

use native_tls::{
    TlsConnector,
    TlsStream,
};

use crate::{
    command::{client, server, Command},
    limits,
    message::{Message, ToMessage},
    END_OF_MESSAGE,
    utils::Defaults,
};

#[derive(Debug)]
pub enum Port {
    Secure(u16),
    Insecure(u16),
}

impl Port {
    pub fn secure(&self) -> bool {
        match self {
            Port::Secure(_) => true,
            _ => false,
        }
    }

    pub fn insecure(&self) -> bool {
        !self.secure()
    }

    pub fn port(&self) -> u16 {
        if let Port::Secure(port) | Port::Insecure(port) = self {
            return *port
        }

        unreachable!()
    }
}

impl Defaults for Port {
    fn defaults() -> Vec<Self> {
        vec![
            //Port::Insecure(194), // official. requires root privileges, so I'll ignore it..
            Port::Insecure(6667), // in-official

            Port::Secure(6697), // official
            Port::Secure(7000), // in-official
        ]
    }
}

impl Into<u16> for Port {
    fn into(self) -> u16 {
        self.port()
    }
}

type MessageQueue<C> = Vec<Message<C>>;
type SecureTcpStream = TlsStream<TcpStream>;

#[derive(Debug)]
enum InnerStream {
    Insecure(TcpStream),
    Secure(SecureTcpStream),
}

impl InnerStream {
    pub fn secure(&self) -> bool {
        match self {
            InnerStream::Insecure(_) => false,
            InnerStream::Secure(_) => true,
        }
    }

    pub fn insecure(&self) -> bool {
        !self.secure()
    }

    pub fn tcp(&self) -> &TcpStream {
        match self {
            InnerStream::Insecure(ref stream) => stream,
            InnerStream::Secure(ref secure_stream) => secure_stream.get_ref()
        }
    }

    pub fn tcp_mut(&mut self) -> &mut TcpStream {
        match self {
            InnerStream::Insecure(ref mut stream) => stream,
            InnerStream::Secure(ref mut secure_stream) => secure_stream.get_mut()
        }
    }

    pub fn tls(&self) -> Option<&SecureTcpStream> {
        match self {
            InnerStream::Secure(ref secure_stream) => Some(secure_stream),
            _ => None,
        }
    }

    pub fn tls_mut(&mut self) -> Option<&mut SecureTcpStream> {
        match self {
            InnerStream::Secure(ref mut secure_stream) => Some(secure_stream),
            _ => None,
        }
    }
}

impl std::io::Read for InnerStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        match self {
            InnerStream::Insecure(ref mut stream) => stream.read(buf),
            InnerStream::Secure(ref mut secure_stream) => secure_stream.read(buf),
        }
    }
}

impl std::io::Write for InnerStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        match self {
            InnerStream::Insecure(ref mut stream) => stream.write(buf),
            InnerStream::Secure(ref mut secure_stream) => secure_stream.write(buf),
        }
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        match self {
            InnerStream::Insecure(ref mut stream) => stream.flush(),
            InnerStream::Secure(ref mut secure_stream) => secure_stream.flush(),
        }
    }
}

#[derive(Debug)]
pub struct Stream<C> {
    inner_stream: RefCell<InnerStream>,
    message_queue: RefCell<MessageQueue<C>>,
    buffer: RefCell<String>,
}

pub type ClientStream = Stream<client::Command>;
pub type ServerStream = Stream<server::Command>;

impl<C> Stream<C>
    where
        C: Command,
{
    pub fn connect(host: &str, port: Port, timeout: Option<std::time::Duration>) -> Result<Self, Box<Error>> {
        let mut tcp_stream = if let Some(timeout) = timeout {
            let mut last_err = None;
            let mut stream = None;

            for addr in (host, port.port()).to_socket_addrs()? {
                match TcpStream::connect_timeout(&addr, timeout) {
                    Ok(_stream) => stream = Some(_stream),
                    Err(e) => last_err = Some(Box::new(e))
                }
            }

            if stream.is_none() {
                return Err(last_err.unwrap())
            }

            stream.unwrap()
        } else {
            TcpStream::connect((host, port.port()))?
        };

        let mut stream = if port.secure() {
            let mut tls_stream = TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .min_protocol_version(Some(native_tls::Protocol::Tlsv12))
                .build()?
                .connect(host, tcp_stream)?;

            InnerStream::Secure(tls_stream)
        } else {
            InnerStream::Insecure(tcp_stream)
        };

        let _ = stream.tcp_mut().set_nodelay(true);
        let _ = stream.tcp_mut().set_nonblocking(true);

        Ok(Stream {
            inner_stream: stream.into(),
            message_queue: MessageQueue::new().into(),
            buffer: format!("").into(),
        })
    }

    pub fn close(self) {
        if let Some(tls_stream) = self.inner_stream.borrow_mut().tls_mut() {
            tls_stream.shutdown().expect("shutting down tls stream")
        }

        let _ = self.inner_stream.borrow_mut().tcp_mut().shutdown(Shutdown::Both);
    }

    pub fn set_timeout(&self, timeout: Duration) {
        let _ = self.inner_stream.borrow_mut().tcp_mut().set_read_timeout(Some(timeout));
    }

    fn read_some(&self) -> Result<(), Box<Error>> {
        if self.buffer.borrow().len() < limits::MESSAGE {
            let mut read_buffer = [0u8; 16 * limits::MESSAGE];

            match self.inner_stream.borrow_mut().read(&mut read_buffer) {
                Ok(size) => {
                    let enc = String::from_utf8_lossy(&read_buffer[..size]).to_string();
                    self.buffer.borrow_mut().push_str(&enc);
                }

                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(())
                }

                Err(e) => return Err(e.into())
            }
        }

        loop {
            let line_end = if let Some(_line_end) = self.buffer.borrow().find(END_OF_MESSAGE) {
                _line_end
            } else {
                break;
            };

            let line = self.buffer.borrow_mut().drain(..line_end + 2).collect::<String>();

            if let Ok(message) = Message::try_from(line.as_str()) {
                self.message_queue.borrow_mut().push(message);

                // TODO:    use this instead of former call after implementing all commands so it's
                //          clear that the message received was malformated.
                //
                //                self.tcp_stream.read_exact(vec![0; line_end + 2].as_mut_slice())?;
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn total(&self) -> usize {
        self.message_queue.borrow().len()
    }

    pub fn read(&self) -> Result<Option<Message<C>>, Box<Error>> {
        if self.message_queue.borrow().is_empty() {
            self.read_some()?;
        }

        if self.message_queue.borrow().is_empty() {
            Ok(None)
        } else {
            let message = self.message_queue.borrow_mut().remove(0);

            let secure = if self.inner_stream.borrow().secure() {
                "secure"
            } else {
                "insecure"
            };

            println!("IN ({})>  {:#?}", secure, message.command());

            Ok(Some(message))
        }
    }

    pub fn send<T>(&self, msg_or_cmd: T) -> Result<&Self, Box<Error>>
                   where
                       T: ToMessage<C> + std::fmt::Debug,
    {
        let secure = if self.inner_stream.borrow().secure() {
            "secure"
        } else {
            "insecure"
        };

        println!("OUT ({})> {:#?}", secure, msg_or_cmd);

        self.inner_stream
            .borrow_mut()
            .write(msg_or_cmd.into_message().to_string().as_bytes())?;

        Ok(self)
    }

    pub fn iter(&self) -> Iter<C> {
        Iter { stream: self }
    }
}

pub struct Iter<'a, C> {
    stream: &'a Stream<C>
}

impl<'a, C> Iterator for Iter<'a, C>
    where
        C: Command,
{
    type Item = Option<Message<C>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.stream.read() {
            Ok(message) => Some(message),
            _ => None,
        }
    }
}
