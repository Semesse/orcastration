// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct MessageState;
// impl MessageState {
//     pub const CREATED: u64 = 0;
//     pub const ORIGINATE_SENT: u64 = 114;
//     pub const ORIGINATE_RECEIVED: u64 = 514;
//     pub const ACK_SENT: u64 = 1919;
//     pub const ACK_RECEIVED: u64 = 810;
// }
use num_enum::IntoPrimitive;

use crate::helper::get_current_nanos;

#[derive(Debug, Clone, Copy, Eq, PartialEq, IntoPrimitive)]
#[repr(u64)]
pub enum MessageState {
    Created = 0,
    ClientStart = 114,
    ClientEnd = 514,
    ServerStart = 1919,
    ServerEnd = 810,
}

// #[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Message {
    pub state: MessageState,
    // local time
    pub originate_timestamp: u128,
    pub finish_timestamp: u128,
    // remote time
    pub receive_timestamp: u128,
    pub ack_timestamp: u128,
}

#[derive(Debug, Clone, Copy)]
pub struct MessageBuf {
    pub buf: [u8; std::mem::size_of::<Message>()],
}

impl MessageBuf {
    pub fn as_bytes_mut(&mut self) -> &mut [u8; std::mem::size_of::<Message>()] {
        &mut self.buf
    }

    pub fn as_message_mut(&mut self) -> &mut Message {
        unsafe { &mut *(self.buf.as_mut_ptr() as *mut Message) }
    }

    pub fn new() -> MessageBuf {
        MessageBuf {
            buf: unsafe { std::mem::zeroed() },
        }
    }

    pub fn from_message(m: &Message) -> MessageBuf {
        let mut mb = MessageBuf::new();
        *mb.as_message_mut() = *m;
        mb
    }

    pub fn from_buf(buf: &[u8; std::mem::size_of::<Message>()]) -> MessageBuf {
        let mut mb = MessageBuf::new();
        *mb.as_bytes_mut() = *buf;
        mb
    }

    pub fn get_state(&mut self) -> MessageState {
        self.as_message_mut().state
    }

    pub fn set_state(&mut self, state: MessageState) {
        let m = self.as_message_mut();
        m.state = state;
        match state {
            MessageState::Created => (),
            MessageState::ClientStart => m.originate_timestamp = get_current_nanos(),
            MessageState::ClientEnd => m.receive_timestamp = get_current_nanos(),
            MessageState::ServerStart => m.ack_timestamp = get_current_nanos(),
            MessageState::ServerEnd => m.finish_timestamp = get_current_nanos(),
        }
    }
}

// impl Message {
//     pub fn create() -> Message {
//         return Message {
//             state: MessageState::Created,
//             originate_timestamp: 0,
//             receive_timestamp: 0,
//             ack_timestamp: 0,
//             finish_timestamp: 0,
//         };
//     }
//     pub fn from_buf(buf: &[u8; std::mem::size_of::<Message>()]) -> Message {
//         let mut m: Message = Message::create();
//         // unsafe {
//         //     (&mut m).clone_from(&std::mem::transmute_copy::<
//         //         [u8; std::mem::size_of::<Message>()],
//         //         Message,
//         //     >(&buf));
//         // }
//         // unsafe {
//         //     (&mut m).clone_from(&(buf as Message));
//         //     std::ptr::write(
//         //         &mut m,
//         //         std::mem::transmute_copy::<[u8; std::mem::size_of::<Message>()], Message>(&buf), // std::mem::size_of::<Message>(),
//         //     )
//         // }
//         // let m = unsafe {
//         //     std::mem::transmute_copy::<[u8; std::mem::size_of::<Message>()], Message>(&buf)
//         // }
//         // .clone();
//         // println!("msg {:?}", m);
//         // m
//         // unsafe {
//         //     std::ptr::read_unaligned(buf, std::mem::transmute(&m), std::mem::size_of::<Message>())
//         // }
//         // unsafe { std::ptr::copy(buf, std::mem::transmute(&m), std::mem::size_of::<Message>()) }
//         // m
//     }

//     pub fn write_buf(self, buf: &mut [u8; std::mem::size_of::<Message>()]) {
//         unsafe {
//             std::ptr::copy(
//                 &self as *const Message,
//                 buf.as_ptr() as *mut Message,
//                 std::mem::size_of::<Message>(),
//             )
//         }
//     }
// }
