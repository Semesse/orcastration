use crossbeam::channel::{self};
use log::warn;
use std::net::SocketAddr;

use std::thread;
use std::{io::Error, net::UdpSocket};

use crate::helper::get_current_nanos;
use crate::protocol::{Message, MessageBuf, MessageState};

pub struct Server {
    pub socket: UdpSocket,
}

impl Server {
    pub fn start(self) -> Result<channel::Receiver<(MessageBuf, SocketAddr)>, Error> {
        let (s, r) = channel::unbounded();
        let socket = self.socket.try_clone().expect("failed to clone socket");

        println!("{:?}", s);

        thread::spawn(move || {
            let mut buf = Box::new([0; std::mem::size_of::<Message>()]);
            loop {
                match socket.recv_from(buf.as_mut_slice()) {
                    Ok((_size, addr)) => {
                        let mut mb = MessageBuf { buf: *buf.clone() };
                        mb.set_state(MessageState::ClientEnd);
                        s.send((mb, addr)).unwrap();
                    }
                    Err(e) => {
                        warn!("err reading socket {:?}", e)
                    }
                }
            }
        });

        // thread::spawn(move || {
        //     let mut buf = Box::new([0; std::mem::size_of::<Message>()]);
        //     loop {
        //         match socket.recv_from(&mut buf.as_mut_slice()) {
        //             Ok((size, addr)) => {
        //                 let now = get_current_nanos().unwrap();
        //                 info!("size {} {:?}", size, buf);
        //                 let m: &mut Message =
        //                     unsafe { std::mem::transmute(buf.as_ptr() as *const &mut Message) };
        //                 m.receive_timestamp = now;
        //                 info!("{:?}", m);

        //                 if m.state == MessageState::OriginateSent {
        //                     s.send(m.clone()).ok();
        //                 }
        //             }
        //             Err(e) => {
        //                 warn!("err reading socket {:?}", e)
        //             }
        //         }
        //     }
        // });

        Ok(r)
    }
}
