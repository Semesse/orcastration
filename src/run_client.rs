#![feature(integer_atomics)]

use cpal::traits::{DeviceTrait, HostTrait};
use log::info;
use rodio::source::SineWave;
use rodio::{source::Source, Decoder, OutputStream};
use rodio::{Sample, Sink};
use std::convert::TryInto;
use std::fs::File;
use std::io::BufReader;
use std::net::UdpSocket;
use std::time::Duration;

use orcar::{
    client::{Client, ClientOptions},
    helper::get_current_nanos,
};

fn skip_samples<I>(input: &mut I, n: usize)
where
    I: Source,
    I::Item: Sample,
{
    for _ in 0..n {
        input.next();
        }
    }
}

pub fn main() {
    env_logger::init();
    let socket = UdpSocket::bind("0.0.0.0:1331").expect("failed to bind socket");
    socket
        .connect("127.0.0.1:1337")
        .expect("failed to connect server");
    let mut client = Client::new(socket);
    let synced_offset = client.synced_offset.clone();
    let thread = std::thread::spawn(move || {
        // let devs = cpal::default_host()
        //     .devices()
        //     .unwrap()
        //     .map(|x| x.name().unwrap())
        //     .collect::<String>();
        println!(
            "{:?}",
            cpal::default_host()
                .default_output_device()
                .unwrap()
                .default_output_config()
                .unwrap()
        );

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // let sink = Sink::try_new(&stream_handle).unwrap();

        // // Add a dummy source of the sake of the example.
        // let source = SineWave::new(440)
        //     .take_duration(Duration::from_secs_f32(1f32))
        //     .amplify(0.20);
        // sink.append(source);
        // sink.sleep_until_end();
        std::thread::sleep(std::time::Duration::from_secs(3));
        // let (_stream, stream_handle) =
        //     OutputStream::try_from_device(&cpal::default_host().default_output_device().unwrap())
        //         .unwrap();
        let args = std::env::args().collect::<Vec<String>>();
        let path = std::path::Path::new(&args[1]);
        info!("{:?}", path);
        let file = BufReader::new(File::open(path).unwrap());
        let duration = mp3_duration::from_path(path).unwrap();
        let mut source = Decoder::new(file).unwrap().repeat_infinite();
        // let sink = Sink:x:try_new(&stream_handle).unwrap();
        let now = get_current_nanos()
            .wrapping_sub(synced_offset.load(std::sync::atomic::Ordering::Relaxed) as u128);
        let skip_duration_nanos = now % duration.as_nanos();
        // info!(
        //     "skip {} {} {} {} {:?}",
        //     synced_offset.load(std::sync::atomic::Ordering::Relaxed),
        //     get_current_nanos(),
        //     now,
        //     duration.as_nanos(),
        //     Duration::from_nanos(skip_duration_nanos as u64)
        // );
        // keep silent until the start of the song
        // let mut source = source.delay(Duration::from_nanos(skip_duration.try_into().unwrap()));
        // or skip first x frames, to make it play immediately
        let skip_samples_count: usize = (skip_duration_nanos)
            .wrapping_mul(source.sample_rate().try_into().unwrap())
            .wrapping_div(1_000_000_000)
            .wrapping_mul(source.channels().try_into().unwrap())
            .try_into()
            .unwrap();
        println!("skip {} {}", skip_duration_nanos, skip_samples_count);
        skip_samples(&mut source, skip_samples_count);

        let now = get_current_nanos()
            .wrapping_sub(synced_offset.load(std::sync::atomic::Ordering::Relaxed) as u128);
        let skip_duration_nanos_1 = now % duration.as_nanos();

        let skip_samples_count_1: usize = (skip_duration_nanos_1 - skip_duration_nanos)
            .wrapping_mul(source.sample_rate().try_into().unwrap())
            .wrapping_div(1_000_000_000)
            .wrapping_mul(source.channels().try_into().unwrap())
            // .wrapping_sub(skip_samples_count.try_into().unwrap())
            .try_into()
            .unwrap();
        println!("skip {} {}", skip_duration_nanos, skip_samples_count_1);
        skip_samples(&mut source, skip_samples_count_1);
        // let s = skip::skip(source, skip_samples_count);
        stream_handle.play_raw(source.convert_samples());
        std::thread::sleep(Duration::MAX);
    });
    client.start(ClientOptions {
        sync_interval_milis: 100,
    });
    thread.join().unwrap();
    // loop {
    //     // let m = Message {
    //     //     state: MessageState::ORIGINATE_SENT,
    //     //     originate_timestamp: get_current_nanos(),
    //     //     receive_timestamp: 0,
    //     //     ack_timestamp: 0,
    //     //     finish_timestamp: 0,
    //     // };
    //     // let mut mb = MessageBuf::from_message(&m);

    //     // let mut mb = MessageBuf::new();
    //     // mb.set_state(MessageState::OriginateSent);
    //     // socket.send(mb.as_bytes_mut()).expect("failed to send");
    //     // match socket.recv_from(buf.as_mut_slice()) {
    //     //     Ok((_size, _addr)) => {
    //     //         let mut m = MessageBuf::from_buf(&*buf);
    //     //         debug!("{:?}", m.as_message_mut());
    //     //     }
    //     //     Err(e) => {
    //     //         warn!("{}", e)
    //     //     }
    //     // }
    // }
}
