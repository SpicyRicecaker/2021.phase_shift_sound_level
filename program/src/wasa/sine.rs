use std::{collections::VecDeque, f64::consts::PI};

// This is a simple structure
#[derive(Debug, Clone, Copy)]
pub struct SineGenerator {
    time: f64,
    pub freq: f64,
    delta_t: f64,
    amplitude: f64,
}

impl SineGenerator {
    pub fn new(freq: f64, fs: f64, amplitude: f64) -> Self {
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
    pub fn distance(&self) -> f64 {
        343. / self.freq
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

/// SineGenerator wrapper with phase shifts
#[derive(Debug, Clone)]
pub struct SineGeneratorCached {
    // Stores all the values (1 / fs) * x depending on phase shift amount
    stack: VecDeque<f32>,
    sine_generator: SineGenerator,
}

impl SineGeneratorCached {
    pub fn new(phase_shift: f64, mut sine_generator: SineGenerator) -> Self {
        let mut stack = VecDeque::new();

        // Difference in time
        let difference_time = phase_shift * (1. / sine_generator.freq);
        // Keep adding delta_ts until we reach past the phase shift
        loop {
            if sine_generator.time >= difference_time {
                // dbg!(sine_generator.time, phase_shift);
                // dbg!(&stack);
                break;
            }
            stack.push_back(sine_generator.next().unwrap());
        }

        SineGeneratorCached {
            stack,
            sine_generator,
        }
    }
}

impl Iterator for SineGeneratorCached {
    type Item = (f32, f32);
    /// Returns the output for the front wave, then the lagging wave, respectively
    fn next(&mut self) -> Option<(f32, f32)> {
        let leading = self.sine_generator.next().unwrap();
        self.stack.push_back(leading);
        let lagging = self.stack.pop_front().unwrap();
        // dbg!(leading, lagging);
        Some((leading, lagging))
    }
}
