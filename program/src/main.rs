use program::wasa::{
    playback_buffer,
    sine::{SineGenerator, SineGeneratorCached},
    wasa,
};
/// Checkout https://docs.rs/rodio/0.14.0/rodio/
use std::{
    io::Write,
    sync::{atomic::AtomicBool, Arc},
    thread::{self, JoinHandle},
};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};
use std::sync::atomic::Ordering;
// #[macro_use]
// mod macros;
// mod test;

struct Config {
    // Frequency, in Hz
    // Shouldn't change
    frequency: u32,
    // Fractional amount that describes the wavelength we're changing
    increment: f32,
    current_increment: f32,
}

// #[derive(Debug)]
// enum State {
//     Playing,
//     Paused,
// }
fn start_playback(
    running: &Arc<AtomicBool>,
    join_handle: &mut Option<JoinHandle<()>>,
    phase_shift: f64,
    sine_generator: SineGenerator,
) {
    // set ourselves to running
    running.store(true, Ordering::Relaxed);
    let running = Arc::clone(running);
    // Clone ourselves a sine generator
    // ...And make a clone the sine cached generator
    // spawn new thread
    *join_handle = Some(thread::spawn(move || {
        let mut gen = SineGeneratorCached::new(phase_shift, sine_generator);

        // Code specific for audio api
        // Wasapi init
        let playback = wasa();
        // Enable playback stream
        playback.audio_client.start_stream().unwrap();

        while running.load(Ordering::Relaxed) {
            playback_buffer(&playback, &mut gen);
        }
    }));
}

fn stop_playback(running: &Arc<AtomicBool>, join_handle: &mut Option<JoinHandle<()>>) {
    running.store(false, Ordering::Relaxed);
    join_handle.take().map(JoinHandle::join);
}

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let running: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    // let mut config = Config {
    //     frequency: 440,
    //     increment: 1.0 / 20.0,
    //     current_increment: 0.328231,
    // };

    // Data for playback
    // Phase shift
    let mut phase_shift: f64 = 0.0;
    // Source data
    let mut sine_generator: SineGenerator = SineGenerator::new(250.0, 48000.0, 0.1);
    // const gen: SineGeneratorCached = SineGeneratorCached::new(phase_shift, sine_generator);
    let mut join_handle: Option<JoinHandle<()>> = None;

    // Today I realized crossterm doesn't magically solve thread problems for you
    // The only way to have continous user input along with continuous audio output
    // is to have them in separate threads
    // Loop for accepting user input (in raw mode)
    loop {
        let menu = format!(
            r#"
                ' ' - toggle audio playback
                'left' - decrease wavelength delay by 0.05λ
                'right' - increase wavelength delay by 0.05λ
                'up' - increase frequency by 10 Hz
                'down' - decrease frequency by 10 Hz
                'q' - quit
                frequency: {:.2}Hz
                phase shift: {:.2}% (negatives don't work)
                1λ: {:.2}m
            "#,
            sine_generator.freq,
            phase_shift,
            sine_generator.distance()
        );

        // clear screen
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        // Manually writes the MENU static string to stdout
        for line in menu.split('\n') {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Left => {
                    stop_playback(&running, &mut join_handle);
                    phase_shift -= 0.05;
                    start_playback(&running, &mut join_handle, phase_shift, sine_generator);
                }
                KeyCode::Right => {
                    stop_playback(&running, &mut join_handle);
                    phase_shift += 0.05;
                    start_playback(&running, &mut join_handle, phase_shift, sine_generator);
                }
                KeyCode::Up => {
                    stop_playback(&running, &mut join_handle);
                    sine_generator.freq += 10.0;
                    start_playback(&running, &mut join_handle, phase_shift, sine_generator);
                }
                KeyCode::Down => {
                    stop_playback(&running, &mut join_handle);
                    sine_generator.freq -= 10.0;
                    start_playback(&running, &mut join_handle, phase_shift, sine_generator);
                }
                KeyCode::Char(c) => match c {
                    ' ' => {
                        // If we're not running start
                        if !running.load(Ordering::Relaxed) {
                            start_playback(&running, &mut join_handle, phase_shift, sine_generator);
                        } else {
                            // Otherwise stop playback
                            stop_playback(&running, &mut join_handle);
                        }
                    }
                    'q' => break,
                    _ => {}
                },
                _ => {}
            }
        };
    }

    // Release cursor
    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    terminal::disable_raw_mode()
}

// Takes in key code and does something with it
pub fn read_char() -> Result<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        })) = event::read()
        {
            return Ok(c);
        }
    }
}

pub fn buffer_size() -> Result<(u16, u16)> {
    terminal::size()
}

fn main() -> Result<()> {
    let mut stdout = std::io::stdout();
    run(&mut stdout).unwrap();
    Ok(())
}
