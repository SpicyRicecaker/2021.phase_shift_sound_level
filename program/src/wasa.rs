use std::f64::consts::PI;
use wasapi::*;

use simplelog::*;

// This is a simple structure
struct SineGenerator {
    time: f64,
    freq: f64,
    delta_t: f64,
    amplitude: f64,
}

impl SineGenerator {
    fn new(freq: f64, fs: f64, amplitude: f64) -> Self {
        SineGenerator {
            // initiate time at 0
            time: 0.0,
            // frequency (e.g. 440hz)
            freq,
            // This is the sample rate
            delta_t: 1.0 / fs,
            // Amplitude is probably pretty important
            amplitude,
        }
    }
}

// Seems like wasapi takes in an iterator
impl Iterator for SineGenerator {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        // Add dt (sample rate) to time
        self.time += self.delta_t;
        // Output the percieved frequency
        let output = ((self.freq * self.time * PI * 2.).sin() * self.amplitude) as f32;
        Some(output)
    }
}

// Main loop
pub fn wasa() {
    // Initiate logger
    let _ = SimpleLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_time_format_str("%H:%M:%S%.3f")
            .build(),
    );

    // Source data
    let mut gen = SineGenerator::new(1000.0, 44100.0, 0.1);

    let playback = init_playback();

    start_playback(playback, &mut gen)
}

struct PlayBack {
    // Required by the sound api
    audio_client: AudioClient,
    render_client: AudioRenderClient,
    h_event: Handle,
    // These two variables lean more towards config
    n_block_align: u32,
    channels: usize,
}

fn start_playback(p: PlayBack, gen: &mut SineGenerator) {
    // Start playback
    p.audio_client.start_stream().unwrap();
    loop {
        // I don't see where we can specify channels in here
        let buffer_frame_count = p.audio_client.get_available_space_in_frames().unwrap();

        // So data is essentially 1056 frames x 8 blocks each for double channels, and 1056 frames x 4 blocks each for single channels
        let mut data = vec![0u8; buffer_frame_count as usize * p.n_block_align as usize];
        // Now we iterate over the 1056 frames in terms of their 8 blocks or 4 blocks depending on channel count
        data.chunks_exact_mut(p.n_block_align as usize)
            .for_each(|frame| {
                // Basically, we convert a float, e.g. 1.25 into 4 bytes, and floats are not very intuitively stored as bytes so we can ignore 'em
                // Sample is just the amplitude we get from sin wav
                let sample = gen.next().unwrap();
                let sample_bytes = sample.to_le_bytes();
                // Now, two channels would be [4], [4], one for left and one for right
                // One channel would be [4], one for the single channel together
                // We iterate over them [[4], [4]] or [[4]]
                // We calculate the default value by always taking n_block_align / channels, which is 8 / 2 for double channels and 4 / 1 for single channels, so it's always 4 basically
                let value = frame
                    .chunks_exact_mut(p.n_block_align as usize / p.channels as usize)
                    .next()
                    .unwrap();

                // For each group of four, we append our sample bytes into it
                // But this is weird, because how would we control different volumes for left and right channels?
                for (bufbyte, sinebyte) in value.iter_mut().zip(sample_bytes.iter()) {
                    *bufbyte = *sinebyte;
                }
                // frame.chunks_exact_mut(n_block_align as usize / channels as usize).for_each(|value| {
                //     // For each group of four, we append our sample bytes into it
                //     // But this is weird, because how would we control different volumes for left and right channels?
                //     for (bufbyte, sinebyte) in value.iter_mut().zip(sample_bytes.iter()) {
                //         *bufbyte = *sinebyte;
                //     }
                // });
            });

        trace!("write");
        p.render_client
            .write_to_device(buffer_frame_count as usize, p.n_block_align as usize, &data)
            .unwrap();
        trace!("write ok");
        if p.h_event.wait_for_event(1000).is_err() {
            error!("error, stopping playback");
            p.audio_client.stop_stream().unwrap();
            break;
        }
    }
}

fn init_playback() -> PlayBack {
    // No idea what this does
    initialize_mta().unwrap();
    // two channels
    let channels = 2;
    // getting the default win device
    let device = get_default_device(&Direction::Render).unwrap();
    // No idea what this does
    let mut audio_client = device.get_iaudioclient().unwrap();
    // Sets format of wave
    let desired_format = WaveFormat::new(32, 32, &SampleType::Float, 48000, channels);

    // Check if the desired format is supported.
    // Since we have convert = true in the initialize_client call later,
    // it's ok to run with an unsupported format.
    match audio_client.is_supported(&desired_format, &ShareMode::Shared) {
        Ok(None) => {
            debug!("Device supports format {:?}", desired_format);
        }
        Ok(Some(modified)) => {
            debug!(
                "Device doesn't support format:\n{:#?}\nClosest match is:\n{:#?}",
                desired_format, modified
            )
        }
        Err(err) => {
            debug!(
                "Device doesn't support format:\n{:#?}\nError: {}",
                desired_format, err
            );
        }
    }

    // Blockalign is the number of bytes per frame
    // nBlock align is 8 for two channels
    // nBlock align is 4 for 1 channel
    // So probably, if we want only right hand playback, all we have to do is shift the data into the right 4 frames only?
    let n_block_align = desired_format.get_blockalign();
    debug!("Desired playback format: {:?}", desired_format);

    // Period is distance from crest to crest?
    // Why define it though
    let (def_time, min_time) = audio_client.get_periods().unwrap();
    debug!("default period {}, min period {}", def_time, min_time);

    audio_client
        .initialize_client(
            &desired_format,
            def_time as i64,
            &Direction::Render,
            // Exclusive sharemode is important, we want as low latency as possible (must be < 1 ms)
            &ShareMode::Shared,
            // Not sure if convert is good on latency, will check with disabled later
            true,
        )
        .unwrap();
    debug!("initialized playback");

    // No idea what this does
    let h_event = audio_client.set_get_eventhandle().unwrap();

    // No idea what this does
    let render_client = audio_client.get_audiorenderclient().unwrap();
    PlayBack {
        audio_client,
        render_client,
        h_event,
        n_block_align,
        channels,
    }
}
