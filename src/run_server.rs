use crossbeam::select;
use log::{debug, info};
use std::net::UdpSocket;

use orcar::{
    protocol::{Message, MessageState},
    server::Server,
};

pub fn main() {
    env_logger::init();
    let socket = UdpSocket::bind("127.0.0.1:1337").expect("coult not bind socket");
    let socket_ref = socket.try_clone().expect("failed to clone socket");
    let s = Server { socket };
    let rx = s.start().expect("failed to start server");
    let _buf = Box::new([0; std::mem::size_of::<Message>()]);

    loop {
        select! {
            recv(rx) -> result => {
                match result {
                Ok((mut mb, addr)) => {
                    mb.set_state(MessageState::AckSent);
                    info!("message: {:?} {:?}", mb.as_message_mut(), addr);
                    socket_ref.send_to(mb.as_bytes_mut(), addr).ok();
                    debug!("wait next message");
                }
            Err(_e) => {}
        }}
        }
    }

    // for (mut m, addr) in rx.iter() {
    //     println!("message: {:?} {:?}", m, addr);
    //     let now = get_current_nanos();
    //     match now {
    //         Some(now) => {
    //             m.ack_timestamp = now;
    //             m.write_buf(&mut buf);
    //             // println!("{:?}", &m);
    //             socket_ref.send_to(buf.as_slice(), addr).ok();
    //         }
    //         None => warn!("failed to get time"),
    //     }
    //     println!("wait next message");
    // }
}
