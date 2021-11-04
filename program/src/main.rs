use program::gen_sound;
use rodio::{source::SineWave, Decoder, OutputStream, Sink, Source, SpatialSink};
use std::{fs::File, io::BufReader, thread, time::{self, Duration, Instant}};

/// Checkout https://docs.rs/rodio/0.14.0/rodio/
fn main() {
    // let source = SineWave::new(250).take_duration(Duration::from_secs(30));
    // gen_sound();

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink_left = Sink::try_new(&stream_handle).unwrap();
    let sink_right = Sink::try_new(&stream_handle).unwrap();

    // For some reason, rodio plays left channel as right channel. Don't know why
    // So sink_left will play source_right
    // And sink_right will play source_left
    let file = BufReader::new(File::open("res/sine_right.wav").unwrap());
    // Decode that sound file into a source
    let source_right = Decoder::new(file).unwrap().repeat_infinite();

    let file = BufReader::new(File::open("res/sine_left.wav").unwrap());
    let source_left = Decoder::new(file).unwrap().repeat_infinite();

    let instant = Instant::now();
    // let period = 2272727;
    // let half_period = 1136363;
    let wait_duration = Duration::from_nanos(1136363);

    sink_left.append(source_right);
    loop {
        if instant.elapsed() > wait_duration {
            break;
        }
    }

    sink_right.append(source_left);

    sink_left.sleep_until_end();
}
