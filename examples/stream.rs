extern crate np1th_irc;

use std::thread::yield_now;

use np1th_irc::{command::client::Command::*, stream::{ClientStream, Port}};

fn main() -> Result<(), Box<std::error::Error>> {
    println!("Trying to connect..");

    if let Ok(stream) = ClientStream::connect("irc.freenode.org", Port::Secure(7000), None) {
        println!("Connected..");

        stream
            .send(Nick {
                name: format!("avon1a"),
            })?
            .send(User {
                name: format!("~avon1"),
                real_name: format!("whaterver"),
                modes: Vec::new(),
            })?;

        for message in stream.iter() {
            if message.is_none() {
                yield_now();
                continue;
            }

            let message = message.unwrap();
            println!("{:?}", message);

            match message.command().clone() {
                Ping { server1, server2 } => {
                    stream.send(Pong { server1, server2 })?;
                }

                PrivMsg { .. } => {
                    let target = message.origin().nick().unwrap();

                    stream.send(PrivMsg {
                        text: format!("Hello {}!", target),
                        targets: vec![target.to_string()],
                    })?;
                }

                _ => {}
            }
        }
    } else {
        println!("Unable to connect to remote host");
    }

    Ok(())
}
