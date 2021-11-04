use rodio::{source::SineWave, OutputStream, Sink, Source};
use std::{
    thread,
    time::{self, Duration},
};

/// Checkout https://docs.rs/rodio/0.14.0/rodio/
fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink_left = Sink::try_new(&stream_handle).unwrap();
    // let sink_right = Sink::try_new(&stream_handle).unwrap();

    let source = SineWave::new(250).take_duration(Duration::from_secs(30));
    sink_left.append(source);

    sink_left.sleep_until_end();
}
