use program::wasa::wasa;
/// Checkout https://docs.rs/rodio/0.14.0/rodio/
use std::io::Write;

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

struct Config {
    // Frequency, in Hz
    // Shouldn't change
    frequency: u32,
    // Fractional amount that describes the wavelength we're changing
    increment: f32,
    current_increment: f32,
}

#[derive(Debug)]
enum State {
    Playing,
    Paused,
}

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    // Load sources
    // For some reason, rodio plays left channel as right channel. Don't know why
    // So sink_left will play source_right
    // And sink_right will play source_left

    let mut state = State::Paused;

    let mut config = Config {
        frequency: 440,
        increment: 1.0 / 20.0,
        current_increment: 0.328231,
    };

    // const MENU: &str = r#"Crossterm interactive test
    // Controls:
    // - 'r' - resume audio playback
    // - 'p' - stop audio playback
    // - 'k' - increase wavelength delay by 1/8
    // - 'j' - decrease wavelength delay by 1/8
    // - 'q' - quit
    // "#;

    let mut queue: Vec<String> = vec![];

    loop {
        let menu = format!(
            r#"
    'r' - resume audio playback
    'p' - stop audio playback
    'k' - increase wavelength delay by 1/8
    'j' - decrease wavelength delay by 1/8
    'q' - quit 
    current state: {:#?},
    current freq: {} ,
    increment freq: {} ,
    current increment: {} ,
    "#,
            state, config.frequency, config.increment, config.current_increment
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

        for line in &queue {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            'r' => {
                state = State::Playing;
            }
            'p' => {
                state = State::Paused;
            }
            'k' => println!("bye"),
            'j' => println!("bye"),
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
