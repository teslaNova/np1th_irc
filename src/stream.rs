use std::{
    convert::TryFrom,
    error::Error,
    io::{prelude::*, ErrorKind::WouldBlock},
    net::{Shutdown, TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::{
    command::{client, server, Command},
    limits,
    message::{Message, ToMessage},
    END_OF_MESSAGE,
};

type MessageQueue<C> = Vec<Message<C>>;

pub struct Stream<C> {
    tcp_stream: TcpStream,
    message_queue: MessageQueue<C>,
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
            tcp_stream,
            message_queue: MessageQueue::new(),
        })
    }

    pub fn close(self) {
        self.tcp_stream.shutdown(Shutdown::Both);
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.tcp_stream.set_read_timeout(Some(timeout));
    }

    fn read_some(&mut self) -> Result<(), Box<Error>> {
        let mut read_buffer = [0u8; 16 * limits::MESSAGE];

        loop {
            match self.tcp_stream.peek(&mut read_buffer) {
                Err(ref e) if e.kind() == WouldBlock => return Ok(()),
                Err(e) => return Err(Box::new(e)),
                _ => break,
            }
        }

        let mut buffer = String::from_utf8_lossy(&read_buffer).to_string();

        while let Some(line_end) = buffer.find(END_OF_MESSAGE) {
            let line = buffer.drain(..line_end + 2).collect::<String>();

            self.tcp_stream
                .read_exact(vec![0; line_end + 2].as_mut_slice())?;

            if let Ok(message) = Message::try_from(line.as_str()) {
                self.message_queue.push(message);

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

    pub fn read(&mut self) -> Result<Option<Message<C>>, Box<Error>> {
        if self.message_queue.is_empty() {
            self.read_some()?;
        }

        if self.message_queue.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.message_queue.remove(0)))
        }
    }

    pub fn send<T>(&mut self, msg_or_cmd: T) -> Result<(), Box<Error>>
                   where
                       T: ToMessage<C>,
    {
        self.tcp_stream
            .write(msg_or_cmd.into_message().to_string().as_bytes())?;

        Ok(())
    }
}
