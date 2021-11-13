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
    let sine_generator: SineGenerator = SineGenerator::new(440.0, 48000.0, 0.1);
    // const gen: SineGeneratorCached = SineGeneratorCached::new(phase_shift, sine_generator);
    let mut join_handle: Option<JoinHandle<()>> = None;

    // Create a sender reciever to control audio thread execution
    // let (tx, rx) = mpsc::channel();

    // Today I realized crossterm doesn't magically solve thread problems for you
    // The only way to have continous user input along with continuous audio output
    // is to have them in separate threads
    // Loop for accepting user input (in raw mode)
    loop {
        //     let menu = format!(
        //         r#"
        // 'r' - resume audio playback
        // 'p' - stop audio playback
        // 'k' - increase wavelength delay by 1/8
        // 'j' - decrease wavelength delay by 1/8
        // 'q' - quit
        // current state: {:#?},
        // current freq: {} ,
        // increment freq: {} ,
        // current increment: {} ,
        // "#,
        //         running, config.frequency, config.increment, config.current_increment
        //     );

        //     // clear screen
        //     queue!(
        //         w,
        //         style::ResetColor,
        //         terminal::Clear(ClearType::All),
        //         cursor::Hide,
        //         cursor::MoveTo(1, 1)
        //     )?;

        //     // Manually writes the MENU static string to stdout
        //     for line in menu.split('\n') {
        //         queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        //     }

        //     w.flush()?;

        match read_char()? {
            'r' => {
                // Only if we're not current running
                if !running.load(Ordering::Relaxed) {
                    // set ourselves to running
                    running.store(true, Ordering::Relaxed);
                    let running = Arc::clone(&running);
                    // Clone ourselves a sine generator
                    // ...And make a clone the sine cached generator
                    // spawn new thread
                    join_handle = Some(thread::spawn(move || {
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
            }
            'p' => {
                running.store(false, Ordering::Relaxed);
                // tx.send(running);
            }
            'k' => phase_shift += 0.1,
            'j' => phase_shift -= 0.1,
            'q' => break,
            _ => {}
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
