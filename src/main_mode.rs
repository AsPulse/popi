use crate::{
  config::LocalStorage,
  filter::MatchedString,
  finder::{Repo, ReposFinder},
  strings::{
    CLEAR_MESSAGE, CLEAR_MESSAGE_LEN, ERROR_PREFIX, EXIT_MESSAGE, EXIT_MESSAGE_LEN, POPI_HEADER,
  },
  terminal_util::{
    BOTTOM_LEFT_CORNER, BOTTOM_RIGHT_CORNER, HORIZONTAL_LINE, TOP_LEFT_CORNER, TOP_RIGHT_CORNER,
    VERTICAL_LINE,
  },
};
use colored::Colorize;
use crossterm::execute;
use crossterm::{
  cursor,
  terminal::{disable_raw_mode, enable_raw_mode},
};
use crossterm::{
  event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
  queue, style, terminal,
};
use std::{
  cmp,
  io::{stdout, Stdout, Write},
};
use thiserror::Error;

static PINK_COLOR: style::Color = style::Color::Rgb {
  r: 255,
  g: 25,
  b: 94,
};

pub fn call_main_mode(_storage: LocalStorage, finder: ReposFinder) {
  let mut stdout = stdout();
  execute!(
    stdout,
    crossterm::terminal::EnterAlternateScreen,
    crossterm::cursor::Hide,
  )
  .unwrap();

  enable_raw_mode().unwrap();
  let main_mode_process = main_mode(finder);
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
      println!(" {}", "Aborting...".bright_black());
      std::process::exit(130);
    }
    Err(e) => {
      eprintln!(
        " {} {}",
        ERROR_PREFIX.on_red().white().bold(),
        "An error occurred while running popi.".red().bold(),
      );
      eprintln!(" {}{}", format!("{:?}: ", e).bold(), e,);
      std::process::exit(1);
    }
  }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum MainModeError {
  #[error("The terminal width is too narrow.")]
  NotEnoughtTerminalWidth,
  #[error("The terminal height is too narrow.")]
  NotEnoughtTerminalHeight,
  #[error("Failed to get terminal size.")]
  TerminalSizeUnavailable,
  #[error("Failed to read event.")]
  EventReadError,
  #[error("Failed to write to stdout.")]
  StdoutWriteError,
}

enum EscapeBehavior {
  Clear,
  Exit,
}

fn main_mode(finder: ReposFinder) -> Result<Option<Repo>, MainModeError> {
  let mut stdout = stdout();
  let mut keyword = String::new();

  loop {
    let (width, height) = terminal::size().map_err(|_| MainModeError::TerminalSizeUnavailable)?;
    let (width, height) = (width as i16, height as i16);
    let escape_behavior = if keyword.is_empty() {
      EscapeBehavior::Exit
    } else {
      EscapeBehavior::Clear
    };

    queue!(
      stdout,
      terminal::Clear(terminal::ClearType::All),
      cursor::MoveTo(0, 0),
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    let header_text = format!(
      "{}{}{}",
      " ",
      POPI_HEADER,
      safe_repeat(" ", width as isize - POPI_HEADER.len() as isize + 1)?
    );

    queue!(
      stdout,
      cursor::MoveTo(0, 0),
      style::SetBackgroundColor(PINK_COLOR),
      style::SetForegroundColor(style::Color::White),
      style::SetAttribute(style::Attribute::Bold),
      style::Print(header_text),
      style::ResetColor,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    safe_move_to(
      &mut stdout,
      width
        - match escape_behavior {
          EscapeBehavior::Clear => CLEAR_MESSAGE_LEN,
          EscapeBehavior::Exit => EXIT_MESSAGE_LEN,
        },
      4,
      width,
      height,
    )?;
    queue!(
      stdout,
      style::SetForegroundColor(style::Color::DarkGrey),
      style::Print(match escape_behavior {
        EscapeBehavior::Clear => CLEAR_MESSAGE,
        EscapeBehavior::Exit => EXIT_MESSAGE,
      }),
      style::ResetColor,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    safe_move_to(&mut stdout, 0, 1, width, height)?;
    let horizontal_line = safe_repeat(HORIZONTAL_LINE, width as isize - 2)?;
    queue!(
      stdout,
      style::SetForegroundColor(style::Color::Magenta),
      style::Print(TOP_LEFT_CORNER),
      style::Print(&horizontal_line),
      style::Print(TOP_RIGHT_CORNER),
      style::ResetColor,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    safe_move_to(&mut stdout, 0, 2, width, height)?;
    queue!(
      stdout,
      style::SetForegroundColor(style::Color::Magenta),
      style::Print(VERTICAL_LINE),
      style::ResetColor,
      style::Print(" 🔎 "),
      // style::SetAttribute(style::Attribute::Bold),
      style::Print(&keyword),
      style::ResetColor,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    safe_move_to(&mut stdout, width - 1, 2, width, height)?;
    queue!(
      stdout,
      style::SetForegroundColor(style::Color::Magenta),
      style::Print(VERTICAL_LINE),
      style::ResetColor,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    safe_move_to(&mut stdout, 0, 3, width, height)?;
    queue!(
      stdout,
      style::SetForegroundColor(style::Color::Magenta),
      style::Print(BOTTOM_LEFT_CORNER),
      style::Print(&horizontal_line),
      style::Print(BOTTOM_RIGHT_CORNER),
      style::ResetColor,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    let repo_views = height - 5;
    let repos = finder.search_by(&keyword);
    let rendering_repos = &repos[..cmp::min(repo_views as usize, repos.len())];
    rendering_repos.iter().enumerate().for_each(|(i, repo)| {
      let repo_name = &repo.repo.name;
      let (before, matched, after) = split_by_matched(repo_name, &repo.matched_string);
      safe_move_to(&mut stdout, 0, 5 + i as i16, width, height).unwrap();
      queue!(
        stdout,
        style::SetForegroundColor(style::Color::Magenta),
        style::Print(" • "),
        style::ResetColor,
        style::SetForegroundColor(style::Color::White),
        style::Print(before),
        style::SetAttribute(style::Attribute::Bold),
        style::Print(matched),
        style::SetAttribute(style::Attribute::Reset),
        style::Print(after),
        style::ResetColor,
      )
      .unwrap();
    });

    safe_move_to(
      &mut stdout,
      cmp::min(5 + keyword.len(), width as usize - 1) as i16,
      2,
      width,
      height,
    )?;

    queue!(
      stdout,
      cursor::Show,
      cursor::SetCursorStyle::SteadyUnderScore,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;

    stdout
      .flush()
      .map_err(|_| MainModeError::StdoutWriteError)?;
    if let Event::Key(key_event) = event::read().map_err(|_| MainModeError::EventReadError)? {
      match key_event {
        event::KeyEvent {
          code: KeyCode::Char('c'),
          modifiers: KeyModifiers::CONTROL,
          ..
        } => {
          break Ok(None);
        }
        event::KeyEvent {
          code: KeyCode::Esc, ..
        } => match escape_behavior {
          EscapeBehavior::Clear => {
            keyword.clear();
          }
          EscapeBehavior::Exit => {
            break Ok(None);
          }
        },
        KeyEvent {
          code: KeyCode::Backspace,
          ..
        } => {
          keyword.pop();
        }
        KeyEvent {
          code: KeyCode::Char(c),
          ..
        } => {
          keyword.push(c);
        }
        _ => {}
      }
    }
  }
}

fn split_by_matched<'a>(s: &'a str, meta: &MatchedString) -> (&'a str, &'a str, &'a str) {
  (
    &s[..meta.matched_start],
    &s[meta.matched_start..meta.matched_start + meta.matched_length],
    &s[meta.matched_length..],
  )
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

fn safe_move_to(
  stdout: &mut Stdout,
  x: i16,
  y: i16,
  width: i16,
  height: i16,
) -> Result<(), MainModeError> {
  if x >= width || x < 0 {
    return Err(MainModeError::NotEnoughtTerminalWidth);
  }
  if y >= height || y < 0 {
    return Err(MainModeError::NotEnoughtTerminalHeight);
  }
  queue!(stdout, cursor::MoveTo(x as u16, y as u16))
    .map_err(|_| MainModeError::StdoutWriteError)?;
  Ok(())
}
