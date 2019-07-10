use std::{
    convert::TryFrom,
    error::Error,
    io::{prelude::*, ErrorKind::WouldBlock},
    net::{Shutdown, TcpStream, ToSocketAddrs},
    time::Duration,
    cell::RefCell,
};

use crate::{
    command::{client, server, Command},
    limits,
    message::{Message, ToMessage},
    END_OF_MESSAGE,
};

type MessageQueue<C> = Vec<Message<C>>;

#[derive(Debug)]
pub struct Stream<C> {
    tcp_stream: RefCell<TcpStream>,
    message_queue: RefCell<MessageQueue<C>>,
}

pub type ClientStream = Stream<client::Command>;
pub type ServerStream = Stream<server::Command>;

impl<C> Stream<C>
    where
        C: Command,
{
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, Box<Error>> {
        let tcp_stream = TcpStream::connect(addr)?;

        tcp_stream.set_nodelay(true);
        tcp_stream.set_nonblocking(true);

        Ok(Stream {
            tcp_stream: tcp_stream.into(),
            message_queue: MessageQueue::new().into(),
        })
    }

    pub fn close(self) {
        self.tcp_stream.borrow_mut().shutdown(Shutdown::Both);
    }

    pub fn set_timeout(&self, timeout: Duration) {
        self.tcp_stream.borrow_mut().set_read_timeout(Some(timeout));
    }

    fn read_some(&self) -> Result<(), Box<Error>> {
        let mut read_buffer = [0u8; 16 * limits::MESSAGE];

        loop {
            match self.tcp_stream.borrow_mut().peek(&mut read_buffer) {
                Err(ref e) if e.kind() == WouldBlock => return Ok(()),
                Err(e) => return Err(Box::new(e)),
                _ => break,
            }
        }

        let mut buffer = String::from_utf8_lossy(&read_buffer).to_string();

        while let Some(line_end) = buffer.find(END_OF_MESSAGE) {
            let line = buffer.drain(..line_end + 2).collect::<String>();

            self.tcp_stream
                .borrow_mut()
                .read_exact(vec![0; line_end + 2].as_mut_slice())?;

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

            println!("IN>  {:#?}", message.command());

            Ok(Some(message))
        }
    }

    pub fn send<T>(&self, msg_or_cmd: T) -> Result<&Self, Box<Error>>
                   where
                       T: ToMessage<C> + std::fmt::Debug,
    {
        println!("OUT> {:#?}", msg_or_cmd);

        self.tcp_stream
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
