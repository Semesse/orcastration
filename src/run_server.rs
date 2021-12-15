use crossbeam::{channel::tick, select};
use log::{debug, info};
use std::{net::UdpSocket, sync::atomic::AtomicU32, time::Duration};

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
    let ticker = tick(Duration::from_millis(1000));
    let count = AtomicU32::new(0);

    loop {
        select! {
            recv(rx) -> result => {
                match result {
                    Ok((mut mb, addr)) => {
                        count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        mb.set_state(MessageState::ServerStart);
                        // info!("message: {:?} {:?}", mb.as_message_mut(), addr);
                        socket_ref.send_to(mb.as_bytes_mut(), addr).ok();
                    }
                Err(_e) => {}
            }}
            recv(ticker) -> _ => {
                info!("handled {} messages within 1s", count.swap(0, std::sync::atomic::Ordering::Relaxed))
            }
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
