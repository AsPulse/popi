use crate::{
  config::LocalStorage,
  filter::MatchedString,
  finder::{FoundRepo, Repo, ReposFinder},
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
  sync::{mpsc::Sender, Arc},
};
use thiserror::Error;
use tokio::{
  sync::{
    mpsc::{self, Receiver},
    oneshot, Mutex, RwLock,
  },
  task::JoinError,
};

static PINK_COLOR: style::Color = style::Color::Rgb {
  r: 255,
  g: 25,
  b: 94,
};

pub async fn call_main_mode(_storage: LocalStorage, finder: ReposFinder) {
  let mut stdout = stdout();
  execute!(
    stdout,
    crossterm::terminal::EnterAlternateScreen,
    crossterm::cursor::Hide,
  )
  .unwrap();

  enable_raw_mode().unwrap();
  let main_mode_process = main_mode(finder).await;
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
  #[error("Failed to join all workers.")]
  WorkerJoinError,
}

enum EscapeBehavior {
  Clear,
  Exit,
}

struct RenderContext {
  keyword: String,
  escape_behavior: EscapeBehavior,
  repos: Vec<FoundRepo>,
  cursor_show: bool,
}

#[derive(Debug)]
enum ContextChange {
  RenderContextChanged,
  KeywordChanged,
  Finished(Result<Option<Repo>, MainModeError>),
}

#[derive(Clone)]
struct MainModeWorker {
  context: Arc<RwLock<RenderContext>>,
  contextchange_tx: mpsc::Sender<ContextChange>,
}

async fn key_input(
  MainModeWorker {
  context,
  contextchange_tx,
  }: MainModeWorker,
) {
  loop {
    tokio::select! {
      _ = contextchange_tx.closed() => { break; },
      event = tokio::spawn(async { event::read() }) => {
        match event {
          Ok(Ok(Event::Key(key_event))) => {
          match key_event {
              event::KeyEvent {
              code: KeyCode::Char('c'),
              modifiers: KeyModifiers::CONTROL,
              ..
              } => {
                contextchange_tx.send(ContextChange::Finished(Ok(None))).await.unwrap();
                break;
              }
              event::KeyEvent {
              code: KeyCode::Esc, ..
              } => {
                let mut context = context.write().await;
                match context.escape_behavior {
                  EscapeBehavior::Clear => {
                    context.keyword.clear();
                    contextchange_tx.send(ContextChange::KeywordChanged).await.unwrap();
                  }
                  EscapeBehavior::Exit => {
                    contextchange_tx.send(ContextChange::Finished(Ok(None))).await.unwrap();
                    break;
                  }
                }
              }
              KeyEvent {
              code: KeyCode::Backspace,
              ..
              } => {
                let mut context = context.write().await;
                context.keyword.pop();
                drop(context);
                contextchange_tx.send(ContextChange::KeywordChanged).await.unwrap();
              }
              KeyEvent {
              code: KeyCode::Char(c),
              ..
              } => {
                let mut context = context.write().await;
                context.keyword.push(c);
                drop(context);
                contextchange_tx.send(ContextChange::KeywordChanged).await.unwrap();
              }
              _ => {}
            }
          }
          Ok(Ok(_)) => { }
          Ok(Err(_)) | Err(_) => {
            contextchange_tx.send(ContextChange::Finished(Err(MainModeError::EventReadError))).await.unwrap();
            break;
          }
        }
      }
    }
  }
}

async fn keyword_change(
  finder: ReposFinder,
  mut keywordchange_rx: Receiver<String>,
  MainModeWorker {
  context,
  contextchange_tx,
  }: MainModeWorker
) {
  loop {
    tokio::select! {
      _ = contextchange_tx.closed() => {
        break;
      }
      Some(keyword) = keywordchange_rx.recv() => {
        {
          let mut context = context.write().await;
          context.escape_behavior = if keyword.is_empty() {
            EscapeBehavior::Exit
          } else {
            EscapeBehavior::Clear
          };
        }
        contextchange_tx.send(ContextChange::RenderContextChanged).await.unwrap();
        let repos = if keyword.is_empty() {
          vec![]
        } else {
          finder.search_by(&keyword)
        };
        {
          let mut context = context.write().await;
          context.repos = repos;
        }
        contextchange_tx.send(ContextChange::RenderContextChanged).await.unwrap();
      }
    }
  }
}

async fn main_mode(finder: ReposFinder) -> Result<Option<Repo>, MainModeError> {
  let (contextchange_tx, mut contextchange_rx) = mpsc::channel::<ContextChange>(20);
  let (keywordchange_tx, keywordchange_rx) = mpsc::channel::<String>(20);

  let shared_context = Arc::new(RwLock::<RenderContext>::new(RenderContext {
    keyword: String::new(),
    escape_behavior: EscapeBehavior::Clear,
    repos: vec![],
    cursor_show: false,
  }));

  let worker = MainModeWorker {
    context: shared_context.clone(),
    contextchange_tx,
  };

  let keyword_change_worker =
    tokio::spawn(keyword_change(finder, keywordchange_rx, worker.clone()));
  let key_input_worker = tokio::spawn(key_input(worker.clone()));

  worker
    .contextchange_tx
    .send(ContextChange::RenderContextChanged)
    .await
    .unwrap();
  let result = loop {
    match contextchange_rx.recv().await.unwrap() {
      ContextChange::RenderContextChanged => {
        let context = shared_context.read().await;
        if let Err(e) = render(&context) {
          break Err(e);
        };
      }
      ContextChange::KeywordChanged => {
        let keyword = {
          let context = shared_context.read().await;
          context.keyword.clone()
        };
        keywordchange_tx.send(keyword).await.unwrap();
      }
      ContextChange::Finished(result) => {
        break result;
      }
    }
  };
  contextchange_rx.close();
  tokio::join!(keyword_change_worker, key_input_worker)
    .0
    .map_err(|_| MainModeError::WorkerJoinError)?;
  result
}

fn render(context: &RenderContext) -> Result<(), MainModeError> {
  let mut stdout = stdout();
  let (width, height) = terminal::size().map_err(|_| MainModeError::TerminalSizeUnavailable)?;
  let (width, height) = (width as i16, height as i16);

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
      - match context.escape_behavior {
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
    style::Print(match context.escape_behavior {
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
    style::Print(&context.keyword),
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
  let rendering_repos = &context.repos[..cmp::min(repo_views as usize, context.repos.len())];
  rendering_repos.iter().enumerate().for_each(|(i, repo)| {
    safe_move_to(&mut stdout, 0, 5 + i as i16, width, height).unwrap();
    let (before, bold, after) = split_by_matched(&repo.repo.name, &repo.matched_string);
    queue!(
      stdout,
      style::SetForegroundColor(style::Color::Magenta),
      style::Print(" • "),
      style::ResetColor,
      style::SetForegroundColor(style::Color::White),
      style::Print(before),
      style::SetAttribute(style::Attribute::Bold),
      style::Print(bold),
      style::SetAttribute(style::Attribute::Reset),
      style::Print(after),
      style::ResetColor,
    )
    .unwrap();
  });

  safe_move_to(
    &mut stdout,
    cmp::min(5 + context.keyword.len(), width as usize - 1) as i16,
    2,
    width,
    height,
  )?;

  {
    queue!(
      stdout,
      cursor::Show,
      cursor::SetCursorStyle::SteadyUnderScore,
    )
    .map_err(|_| MainModeError::StdoutWriteError)?;
  }

  stdout
    .flush()
    .map_err(|_| MainModeError::StdoutWriteError)?;

  Ok(())
}

fn split_by_matched<'a>(s: &'a str, meta: &MatchedString) -> (&'a str, &'a str, &'a str) {
  (
    &s[..meta.matched_start],
    &s[meta.matched_start..meta.matched_start + meta.matched_length],
    &s[meta.matched_start + meta.matched_length..],
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_split_by_matched() {
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 0,
          matched_length: 1,
          distance: 0,
        }
      ),
      ("", "h", "ello")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 1,
          distance: 0,
        }
      ),
      ("h", "e", "llo")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 2,
          distance: 0,
        }
      ),
      ("h", "el", "lo")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 3,
          distance: 0,
        }
      ),
      ("h", "ell", "o")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 1,
          matched_length: 4,
          distance: 0,
        }
      ),
      ("h", "ello", "")
    );
    assert_eq!(
      split_by_matched(
        "hello",
        &MatchedString {
          matched_start: 0,
          matched_length: 5,
          distance: 0,
        }
      ),
      ("", "hello", "")
    );
  }
}
