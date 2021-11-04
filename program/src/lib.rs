use std::f32::consts::PI;
use std::i16;

pub fn gen_sound() {
    let sample_rate = 44100;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create("res/sine.wav", spec).unwrap();
    (0..20).for_each(|_| {
        (0..sample_rate)
            .map(|x| x as f32 / sample_rate as f32)
            .for_each(|t| {
                let sample = (t * 440.0 * 2.0 * PI).sin();
                let amplitude = i16::MAX as f32;
                writer.write_sample((sample * amplitude) as i16).unwrap();
            });
    });
}
