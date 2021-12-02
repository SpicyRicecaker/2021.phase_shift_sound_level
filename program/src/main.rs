use crossterm::event::KeyModifiers;
use program::wasa::{playback_buffer, sine::SineGeneratorDoubled, wasa};
use std::sync::mpsc::{self, Sender};
/// Checkout https://docs.rs/rodio/0.14.0/rodio/
use std::{
    io::Write,
    thread::{self, JoinHandle},
};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

#[derive(Debug)]
enum Action {
    DeltaFreq(f64),
    DeltaPhaseShift(f64),
    Stop,
}

fn start_playback(mut gen: SineGeneratorDoubled) -> (JoinHandle<()>, Sender<Action>) {
    let (tx, rx) = mpsc::channel();
    // Clone ourselves a sine generator
    // ...And make a clone the sine cached generator
    // spawn new thread
    let join_handle = thread::spawn(move || {
        // Code specific for audio api
        // Wasapi init
        let playback = wasa();
        // Enable playback stream
        playback.audio_client.start_stream().unwrap();

        loop {
            playback_buffer(&playback, &mut gen);
            if let Ok(v) = rx.try_recv() {
                match v {
                    Action::DeltaFreq(d) => gen.freq += d,
                    Action::DeltaPhaseShift(d) => gen.phase_shift += d,
                    Action::Stop => break,
                }
            }
        }
    });
    (join_handle, tx)
}

fn stop_playback(threadc: &mut Option<(JoinHandle<()>, Sender<Action>)>) {
    if let Some((j, s)) = threadc.take() {
        s.send(Action::Stop).unwrap();
        j.join().unwrap();
    }
}

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    // let mut config = Config {
    //     frequency: 440,
    //     increment: 1.0 / 20.0,
    //     current_increment: 0.328231,
    // };

    // Data for playback
    // Source data
    let mut sine_generator: SineGeneratorDoubled = SineGeneratorDoubled::new(264.0, 48000.0, 0.1);
    // const gen: SineGeneratorCached = SineGeneratorCached::new(phase_shift, sine_generator);
    let mut threadc: Option<(JoinHandle<()>, Sender<Action>)> = None;

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
                phase shift: {:.2} / 1 (negatives don't work)
                1λ: {:.2}m
            "#,
            sine_generator.freq,
            sine_generator.phase_shift,
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
        menu.split('\n')
            .try_for_each(|line| queue!(w, style::Print(line), cursor::MoveToNextLine(1)))
            .unwrap();

        w.flush()?;
        if let Event::Key(KeyEvent { code, modifiers }) = event::read()? {
            match code {
                // TODO should probably move it into a struct so we don't have to destructure
                // We're storing and mutating a f32 two times, one in the audio thread and one in main thread.
                KeyCode::Left => {
                    let diff = if modifiers == KeyModifiers::SHIFT {
                        -0.01
                    } else {
                        -0.05
                    };
                    if let Some((_, s)) = &threadc {
                        s.send(Action::DeltaPhaseShift(diff)).unwrap();
                    }
                    sine_generator.phase_shift += diff;
                }
                KeyCode::Right => {
                    let diff = if modifiers == KeyModifiers::SHIFT {
                        0.01
                    } else {
                        0.05
                    };
                    if let Some((_, s)) = &threadc {
                        s.send(Action::DeltaPhaseShift(diff)).unwrap();
                    }
                    sine_generator.phase_shift += diff;
                }
                KeyCode::Up => {
                    let diff = if modifiers == KeyModifiers::SHIFT {
                        1.
                    } else {
                        10.
                    };
                    if let Some((_, s)) = &threadc {
                        s.send(Action::DeltaFreq(diff)).unwrap();
                    }
                    sine_generator.freq += diff;
                }
                KeyCode::Down => {
                    let diff = if modifiers == KeyModifiers::SHIFT {
                        -1.
                    } else {
                        -10.
                    };
                    if let Some((_, s)) = &threadc {
                        s.send(Action::DeltaFreq(diff)).unwrap();
                    }
                    sine_generator.freq += diff;
                }
                KeyCode::Char(c) => match c {
                    ' ' => {
                        // If we're not running start
                        if threadc.is_none() {
                            threadc = Some(start_playback(sine_generator));
                        } else {
                            stop_playback(&mut threadc);
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
