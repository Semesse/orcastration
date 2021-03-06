use crossbeam::{channel, select};
use log::{info, warn};

use std::num::Wrapping;
use std::sync::atomic::AtomicI128;
use std::sync::Arc;
use std::time::Duration;
use std::{net::UdpSocket, thread};

use crate::helper::get_current_nanos;
use crate::protocol::{Message, MessageBuf, MessageState};

pub struct Client {
    pub socket: UdpSocket,
    // offset to local time,
    pub synced_offset: Arc<AtomicI128>,
    min_tx_time: [u128; 3], // duration, start, end
    min_rx_time: [u128; 3],
}

pub struct ClientOptions {
    pub sync_interval_milis: u32,
}

impl Client {
    pub fn new(socket: UdpSocket) -> Self {
        return Client {
            socket,
            synced_offset: Arc::new(AtomicI128::new(0)),
            min_tx_time: [0, 0, 0],
            min_rx_time: [0, 0, 0],
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
        get_current_nanos().wrapping_sub(
            self.synced_offset
                .load(std::sync::atomic::Ordering::Relaxed) as u128,
        )
    }

    fn handle_message(&mut self, mut mb: MessageBuf) {
        if mb.get_state() != MessageState::ServerEnd {
            warn!("received message with wrong state {:?}", mb.get_state());
            return;
        }
        // info!("message {:?}", mb.as_message_mut());
        let m = mb.as_message_mut();
        let tx_time = Wrapping(self.min_tx_time[1]) - Wrapping(self.min_tx_time[0]);
        let rx_time = Wrapping(self.min_rx_time[1]) - Wrapping(self.min_rx_time[0]);
        if Wrapping(m.receive_timestamp) - Wrapping(m.originate_timestamp) < tx_time {
            self.min_tx_time = [tx_time.0, m.originate_timestamp, m.receive_timestamp];
        }
        if Wrapping(m.finish_timestamp) - Wrapping(m.ack_timestamp) < rx_time {
            self.min_rx_time = [rx_time.0, m.ack_timestamp, m.finish_timestamp];
        }

        let tx_time = Wrapping(self.min_tx_time[1]) - Wrapping(self.min_tx_time[0]);
        let rx_time = Wrapping(self.min_rx_time[1]) - Wrapping(self.min_rx_time[0]);
        let synced_time = Wrapping(m.ack_timestamp) + Wrapping((tx_time + rx_time).0 / 2);
        // we assume that offset should not overflow
        // info!(
        //     "synced time {:?} {} {} {}",
        //     m,
        //     (synced_time - Wrapping(m.finish_timestamp)).0 as i128,
        //     synced_time.0,
        //     m.finish_timestamp
        // );
        self.synced_offset.store(
            (Wrapping(m.finish_timestamp) - synced_time).0 as i128,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}
