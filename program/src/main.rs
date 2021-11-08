use program::gen_sound;
use rodio::{source::SineWave, Decoder, OutputStream, Sink, Source, SpatialSink};
use std::{fs::File, io::BufReader, thread, time::{self, Duration, Instant}};

/// Checkout https://docs.rs/rodio/0.14.0/rodio/
fn sound() {

    // let instant = Instant::now();
    // // let period = 2272727;
    // // let half_period = 1136363;
    // // let wait_duration = Duration::from_nanos(1136363);
    // let wait_duration = Duration::from_nanos(0);

    // sink_left.append(source_right);
    // loop {
    //     if instant.elapsed() > wait_duration {
    //         break;
    //     }
    // }

    // sink_right.append(source_left);

    // sink_left.sleep_until_end();
}

// #![allow(clippy::cognitive_complexity)]

use std::io::{self, Write};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

// #[macro_use]
// mod macros;
// mod test;

const MENU: &str = r#"Crossterm interactive test
Controls:
 - 'r' - resume audio playback
 - 'p' - stop audio playback
 - 'k' - increase wavelength delay by 1/8
 - 'j' - decrease wavelength delay by 1/8
 - 'q' - quit 
"#;

struct Config {
    // Frequency, in Hz
    // Shouldn't change
    frequency: u32,
    // Fractional amount that describes the wavelength we're changing 
    increment: f32,
}

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    // Grab sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Generate left and right sinks
    let sink_left = Sink::try_new(&stream_handle).unwrap();
    let sink_right = Sink::try_new(&stream_handle).unwrap();

    // Load sources
    // For some reason, rodio plays left channel as right channel. Don't know why
    // So sink_left will play source_right
    // And sink_right will play source_left
    let file = BufReader::new(File::open("res/sine_right.wav").unwrap());
    // Decode that sound file into a source
    let source_right = Decoder::new(file).unwrap().repeat_infinite();
    let file = BufReader::new(File::open("res/sine_left.wav").unwrap());
    let source_left = Decoder::new(file).unwrap().repeat_infinite();

    loop {
        // clear screen
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        // Manually writes the MENU static string to stdout
        for line in MENU.split('\n') {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            'r' => println!("hello"),
            'p' => println!("bye"),
            'k' => println!("bye"),
            'j' => println!("bye"),
            'q' => break,
            _ => {}
        };
    }

    // execute!(
    //     w,
    //     style::ResetColor,
    //     cursor::Show,
    //     terminal::LeaveAlternateScreen
    // )?;

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
    let mut stdout = io::stdout();
    run(&mut stdout)
}