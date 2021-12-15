use crossbeam::{channel, select};
use log::{debug, info, warn};

use std::time::Duration;
use std::{io::Error, net::UdpSocket, thread};

use crate::helper::get_current_nanos;
use crate::protocol::{Message, MessageBuf, MessageState};

pub struct Client {
    pub socket: UdpSocket,
    // offset to local time
    synced_offset: i128,
    min_tx_time: [u128; 2],
    min_rx_time: [u128; 2],
}

pub struct ClientOptions {
    pub sync_interval_milis: u32,
}

impl Client {
    pub fn new(socket: UdpSocket) -> Self {
        return Client {
            socket,
            synced_offset: 0,
            min_tx_time: [0, 0],
            min_rx_time: [0, 0],
        };
    }

    pub fn start(&mut self, options: ClientOptions) {
        let (s, r) = channel::unbounded();
        let rx = self.socket.try_clone().unwrap();
        let tx = self.socket.try_clone().unwrap();
        // println!("{:?}", s);

        thread::spawn(move || {
            let mut buf = Box::new([0; std::mem::size_of::<Message>()]);
            loop {
                match rx.recv_from(buf.as_mut_slice()) {
                    Ok((size, _addr)) => {
                        let mut mb = MessageBuf::from_buf(&buf);
                        mb.set_state(MessageState::ServerEnd);
                        s.send(mb).ok();
                    }
                    Err(e) => {
                        warn!("err reading socket {:?}", e)
                    }
                }
            }
        });

        let tick = channel::tick(Duration::from_millis(options.sync_interval_milis.into()));
        loop {
            select! {
                recv(tick) -> _ => {
                    let mut mb = MessageBuf::new();
                    mb.set_state(MessageState::ClientStart);
                    tx.send(mb.as_bytes_mut()).ok();
                }
                recv(r) -> mb => {
                    self.handle_message(mb.unwrap());
                }
            }
        }
    }

    pub fn now(&self) -> u128 {
        return (self.synced_offset + get_current_nanos() as i128) as u128;
    }

    fn handle_message(&mut self, mut mb: MessageBuf) {
        if mb.get_state() != MessageState::ServerEnd {
            warn!("received message with wrong state {:?}", mb.get_state());
            return;
        }
        let m = mb.as_message_mut();
        let mut should_update = false;
        if m.receive_timestamp - m.originate_timestamp > self.min_tx_time[1] - self.min_tx_time[0] {
            self.min_tx_time = [m.originate_timestamp, m.receive_timestamp];
            should_update = true;
        }
        if m.finish_timestamp - m.ack_timestamp > self.min_rx_time[1] - self.min_rx_time[0] {
            self.min_rx_time = [m.ack_timestamp, m.finish_timestamp];
            should_update = true;
        }
        if should_update {
            let tx_time = self.min_tx_time[1] - self.min_tx_time[0];
            let rx_time = self.min_rx_time[1] - self.min_rx_time[0];
            let synced_time = m.ack_timestamp + (tx_time + rx_time) / 2;
            // we assume that offset should not overflow
            self.synced_offset = synced_time as i128 - get_current_nanos() as i128;
            info!("synced time {:?}", self.synced_offset);
        }
    }
}
