use std::{
    io::{Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{cursor, execute, style::Print, terminal};

use crate::PER_CHARACTER_DELAY_MS;

/// A struct that serves as the View (V) in MVC.
/// Specializes in displaying the generated typist-art in the terminal.
pub struct View {}

impl View {
    /// Animates the given typist art line by line with a per-character delay.
    pub fn animate(data: &[String]) -> std::io::Result<()> {
        let mut stdout = stdout();

        // clear the terminal.
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )?;

        for y in 0..data.len() {
            for (x, c) in data[y].chars().enumerate() {
                execute!(stdout, cursor::MoveTo((x * 2) as u16, y as u16), Print(c))?;
                stdout.flush()?;
                thread::sleep(Duration::from_millis(PER_CHARACTER_DELAY_MS));
            }
        }

        // move cursor under typist-art after animation
        execute!(stdout, cursor::MoveTo(0, data.len() as u16), cursor::Show)?;

        Ok(())
    }
}
