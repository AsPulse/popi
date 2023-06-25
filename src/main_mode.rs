use std::io::{stdout, Write};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use thiserror::Error;
use colored::Colorize;
use crate::{config::LocalStorage, finder::{ReposFinder, Repo}, strings::{POPI_HEADER, ERROR_PREFIX}};

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

  match main_mode_process {
    Ok(Some(_repo)) => todo!("repo"),
    Ok(None) => {
      println!("{}", "Aborting...".bright_black());
      std::process::exit(130);
    }
    Err(e) => {
      eprintln!(
        " {} {}",
        ERROR_PREFIX.on_red().white().bold(),
        "An error occurred while running popi.".red().bold(),
      );
      eprintln!(
        " {}{}",
        format!(
          "{:?}: ",
          e
        ).bold(),
        e,
      );
      std::process::exit(1);
    }
  }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MainModeError {
  #[error("The terminal width is too narrow.")]
  NotEnoughtTerminalWidth,
  #[error("Failed to get terminal size.")]
  TerminalSizeUnavailable,
  #[error("Failed to read event.")]
  EventReadError,
  #[error("Failed to write to stdout.")]
  StdoutWriteError,
}
fn main_mode(storage: LocalStorage, finder: ReposFinder) -> Result<Option<Repo>, MainModeError> {
  use crossterm::{
    queue,
    terminal, cursor, style,
    event::{
      self, Event,
      KeyCode,
      KeyModifiers,
      KeyEvent,
    },
  };

  let mut stdout = stdout();

  loop {

    let (width, height) = terminal::size().map_err(|_| MainModeError::TerminalSizeUnavailable)?;
    queue!(
      stdout,
      terminal::Clear(terminal::ClearType::All),
      cursor::MoveTo(0, 0),
    ).map_err(|_| MainModeError::StdoutWriteError)?;


    let header_text = format!("{}{}{}", " ", POPI_HEADER, safe_repeat(" ", width as isize - POPI_HEADER.len() as isize + 1)?);
    queue!(
      stdout,
      cursor::MoveTo(0, 0),
      style::SetBackgroundColor(style::Color::Rgb { r: 255, g: 25, b: 94 }),
      style::SetForegroundColor(style::Color::White),
      style::SetAttribute(style::Attribute::Bold),
      style::Print(header_text),
      style::ResetColor,
    ).map_err(|_| MainModeError::StdoutWriteError)?;

    stdout.flush().map_err(|_| MainModeError::StdoutWriteError)?;
    if let Event::Key(key_event) = event::read().map_err(|_| MainModeError::EventReadError)? {
      match key_event {
        KeyEvent {
          code: KeyCode::Esc, ..
        }
        | event::KeyEvent {
          code: KeyCode::Char('c'),
          modifiers: KeyModifiers::CONTROL,
          ..
        } => {
          break Ok(None);
        }
        _ => {}
      }
    }
  }
}
fn safe_repeat(s: &str, n: isize) -> Result<String, MainModeError> {
  if n < 0 {
    return Err(MainModeError::NotEnoughtTerminalWidth);
  }
  if n == 0 {
    return Ok(String::new());
  }
  Ok(s.repeat(n as usize))
}
