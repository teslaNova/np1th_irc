extern crate np1th_irc;

use std::{thread::sleep, time::Duration};

use np1th_irc::{command::client::Command::*, stream::ClientStream};

fn main() {
    println!("Trying to connect..");

    if let Ok(mut stream) = ClientStream::connect("irc.freenode.org:6667") {
        println!("Connected..");

        stream
            .send(Nick {
                name: format!("avon1a"),
            })
            .and_then(|_| {
                stream.send(User {
                    name: format!("~avon1"),
                    real_name: format!("whaterver"),
                    modes: Vec::new(),
                })
            })
            .expect("Something went wrong while trying to register with server");

        loop {
            match stream.read() {
                Ok(Some(message)) => {
                    println!("{:?}", message);

                    match message.command().clone() {
                        Ping { server1, server2 } => {
                            stream.send(Pong { server1, server2 });
                        }

                        PrivMsg { .. } => {
                            let target = message.origin().nick().unwrap();

                            stream.send(PrivMsg {
                                text: format!("Hello {}!", target),
                                targets: vec![target.clone()],
                            });
                        }

                        _ => {}
                    }
                }

                Ok(None) => {
                    // No messages.. idle..
                    sleep(Duration::from_millis(150));
                }

                Err(e) => {
                    eprintln!("Error while reading: {:?}", e);
                    break;
                }
            }
        }
    } else {
        println!("Unable to connect to remote host");
    }
}
