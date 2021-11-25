use log::{debug, warn};
use std::net::UdpSocket;

use orcar::{
    helper::get_current_nanos,
    protocol::{Message, MessageBuf, MessageState},
};

pub fn main() {
    env_logger::init();
    let socket = UdpSocket::bind("127.0.0.1:1331").expect("failed to bind socket");
    let mut buf = Box::new([0; std::mem::size_of::<Message>()]);
    loop {
        // let m = Message {
        //     state: MessageState::ORIGINATE_SENT,
        //     originate_timestamp: get_current_nanos(),
        //     receive_timestamp: 0,
        //     ack_timestamp: 0,
        //     finish_timestamp: 0,
        // };
        // let mut mb = MessageBuf::from_message(&m);
        socket
            .connect("127.0.0.1:1337")
            .expect("failed to connect server");

        let mut mb = MessageBuf::new();
        mb.set_state(MessageState::OriginateSent);
        socket.send(mb.as_bytes_mut()).expect("failed to send");
        match socket.recv_from(buf.as_mut_slice()) {
            Ok((_size, _addr)) => {
                let mut m = MessageBuf::from_buf(&*buf);
                debug!("{:?}", m.as_message_mut());
            }
            Err(e) => {
                warn!("{}", e)
            }
        }
    }
}
