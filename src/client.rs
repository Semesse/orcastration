use crossbeam::channel;
use log::warn;

use std::{io::Error, net::UdpSocket, thread};

use crate::helper::get_current_nanos;
use crate::protocol::{Message, MessageState};

pub struct Client {
    pub socket: UdpSocket,
}

impl Client {
    pub fn start(self) -> Result<channel::Receiver<Message>, Error> {
        let (s, r) = channel::unbounded();
        let socket = self.socket.try_clone().unwrap();
        println!("{:?}", s);

        thread::spawn(move || {
            let mut buf = Box::new([0; std::mem::size_of::<Message>()]);
            loop {
                match socket.recv_from(buf.as_mut_slice()) {
                    Ok((size, _addr)) => {
                        let now = get_current_nanos();
                        println!("size {} {:?}", size, buf);
                        let m: &mut Message =
                            unsafe { std::mem::transmute(buf.as_ptr() as *const &mut Message) };
                        m.ack_timestamp = now;
                        println!("{:?}", m);

                        if m.state == MessageState::OriginateSent {
                            s.send(*m).ok();
                        }
                    }
                    Err(e) => {
                        warn!("err reading socket {:?}", e)
                    }
                }
            }
        });

        Ok(r)
    }
}
