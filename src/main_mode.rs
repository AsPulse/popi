use crate::{config::LocalStorage, finder::ReposFinder, strings::POPI_HEADER};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::{
  error::Error,
  io::{stdout, Write},
};

pub fn call_main_mode(storage: LocalStorage, finder: ReposFinder) {
  use crossterm::execute;

  let mut stdout = stdout();
  execute!(
    stdout,
    crossterm::terminal::EnterAlternateScreen,
    crossterm::cursor::Hide,
  )
  .unwrap();

  enable_raw_mode().unwrap();
  let main_mode_process = main_mode(storage, finder);
  disable_raw_mode().unwrap();

  execute!(
    stdout,
    crossterm::cursor::Show,
    crossterm::terminal::LeaveAlternateScreen,
  )
  .unwrap();

  disable_raw_mode().unwrap();
  main_mode_process.unwrap();
}

fn main_mode(storage: LocalStorage, finder: ReposFinder) -> Result<(), Box<dyn Error>> {
  use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue, style, terminal,
  };

  let mut stdout = stdout();

  loop {
    let (width, height) = terminal::size()?;
    queue!(
      stdout,
      terminal::Clear(terminal::ClearType::All),
      cursor::MoveTo(0, 0),
    )?;

    queue!(
      stdout,
      cursor::MoveTo(0, 0),
      style::SetBackgroundColor(style::Color::Rgb {
        r: 255,
        g: 25,
        b: 94
      }),
      style::SetForegroundColor(style::Color::White),
      style::Print(format!(
        "{}{}{}",
        " ",
        POPI_HEADER,
        " ".repeat(width as usize - POPI_HEADER.len() + 1)
      ),),
      style::ResetColor,
    )?;

    stdout.flush()?;
    if let Event::Key(key_event) = event::read()? {
      match key_event {
        KeyEvent {
          code: KeyCode::Esc, ..
        }
        | event::KeyEvent {
          code: KeyCode::Char('c'),
          modifiers: KeyModifiers::CONTROL,
          ..
        } => {
          break;
        }
        _ => {}
      }
    }
  }

  Ok(())
}
