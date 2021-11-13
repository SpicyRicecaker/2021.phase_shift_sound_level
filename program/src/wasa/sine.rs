use std::f64::consts::PI;

// This is a simple structure
#[derive(Debug, Clone, Copy)]
pub struct SineGeneratorDoubled {
    time: f64,
    pub freq: f64,
    delta_t: f64,
    amplitude: f64,
    pub phase_shift: f64,
}

impl SineGeneratorDoubled {
    pub fn new(freq: f64, fs: f64, amplitude: f64) -> Self {
        SineGeneratorDoubled {
            // initiate time at 0
            time: 0.0,
            // frequency (e.g. 440hz)
            freq,
            // This is the sample rate
            delta_t: 1.0 / fs,
            // Amplitude is probably pretty important
            amplitude,
            // Default phase 0
            phase_shift: 0.0,
        }
    }
    pub fn distance(&self) -> f64 {
        343. / self.freq
    }
}

// Seems like wasapi takes in an iterator
impl Iterator for SineGeneratorDoubled {
    type Item = (f32, f32);
    fn next(&mut self) -> Option<(f32, f32)> {
        // Add dt (sample rate) to time
        self.time += self.delta_t;
        let leading = ((self.freq * self.time * PI * 2.).sin() * self.amplitude) as f32;
        // Recall that y = sin(b(x+c))
        // Period is 2pi / b, so if we have 5 hz * 2pi, then period = 2pi / 10 pi = 1 / 5
        // This means if we want a phase shift of 1/2, we need to do 1/2 / 5, which would be 1/10
        // TODO Not sure if we could / should simplify this expression, or the performance benefits it would give
        let lagging = ((self.freq * (self.time + (self.phase_shift) / self.freq) * PI * 2.).sin()
            * self.amplitude) as f32;
        Some((leading, lagging))
    }
}
