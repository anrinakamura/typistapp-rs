use std::{
    io::{Write, stdout},
    thread,
    time::Duration,
};

use crossterm::{cursor, execute, style::Print, terminal};

pub struct View {}

impl View {
    pub fn run() {
        let data = vec![
            "あ".to_string(),
            "い".to_string(),
            "う".to_string(),
            "え".to_string(),
            "お".to_string(),
            "\n".to_string(),
            "か".to_string(),
            "き".to_string(),
            "く".to_string(),
            "け".to_string(),
            "こ".to_string(),
            "\n".to_string(),
            "さ".to_string(),
            "し".to_string(),
            "す".to_string(),
            "せ".to_string(),
            "そ".to_string(),
            "\n".to_string(),
        ];

        thread::sleep(Duration::from_secs(3));

        if let Err(e) = Self::animate(&data, 6, 3) {
            let _ = execute!(stdout(), cursor::Show);
            log::error!("Error: {}", e);
        }

        log::info!("Animation completed successfully!");
    }

    fn animate(data: &[String], columns: u32, rows: u32) -> std::io::Result<()> {
        let mut stdout = stdout();

        // clear the terminal.
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Hide
        )?;

        // // get terminal width.
        // let (terminal_width, _) = terminal::size()?;
        // let m = (terminal_width / 2) as usize;

        let mut data_cycle = data.iter().cycle();

        'outer: for y in 0..rows {
            for x in 0..columns {
                if let Some(s) = data_cycle.next() {
                    execute!(stdout, cursor::MoveTo((x * 2) as u16, y as u16), Print(s))?;

                    stdout.flush()?;
                    thread::sleep(Duration::from_millis(10));
                } else {
                    break 'outer;
                }
            }
        }

        // move cursor under typist-art after animation
        execute!(stdout, cursor::MoveTo(0, rows as u16), cursor::Show)?;

        Ok(())
    }
}
