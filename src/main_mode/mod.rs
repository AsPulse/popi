mod safe_methods;
mod split_by_matched;
mod worker_cursor_blinker;
mod worker_key_input;
mod worker_keyword_change;

use crate::{
  colors::{BACKGROUND_PINK_COLOR, LIGHTER_PINK_COLOR, PINK_COLOR},
  config::LocalStorage,
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
use crossterm::{queue, style, terminal};
use std::{
  cmp, env,
  io::{stderr, Write},
  sync::Arc,
};
use thiserror::Error;
use tokio::sync::{mpsc, RwLock, RwLockWriteGuard};

use safe_methods::{safe_move_to, safe_repeat};
use split_by_matched::split_by_matched;
use worker_cursor_blinker::cursor_blinker;
use worker_key_input::key_input;
use worker_keyword_change::keyword_change;

pub async fn call_main_mode(_storage: LocalStorage, finder: ReposFinder) {
  let mut stderr = stderr();
  execute!(
    stderr,
    crossterm::terminal::EnterAlternateScreen,
    crossterm::cursor::Hide,
  )
  .unwrap();

  enable_raw_mode().unwrap();
  let main_mode_process = main_mode(finder).await;
  disable_raw_mode().unwrap();

  execute!(
    stderr,
    crossterm::cursor::Show,
    crossterm::terminal::LeaveAlternateScreen,
    crossterm::cursor::SetCursorStyle::DefaultUserShape,
  )
  .unwrap();

  disable_raw_mode().unwrap();

  match main_mode_process {
    Ok(Some(repo)) => {
      let path = repo.path.to_str().unwrap().bold();
      env::set_var("POPI_REPO_PATH", repo.path.to_str().unwrap());
      eprintln!(" {} {}", "Go ahead!".cyan().bold(), path.normal());
      eprintln!(
        " {}",
        "Path to repository was written to stdout.".bright_black(),
      );
      eprintln!();
      println!("{}", repo.path.to_str().unwrap());
      std::process::exit(0);
    }
    Ok(None) => {
      eprintln!(" {}", "Aborting...".bright_black());
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
  #[error("Failed to write to stderr.")]
  StdoutWriteError,
  #[error("Failed to join all workers.")]
  WorkerJoinError,
}

pub(super) enum EscapeBehavior {
  Clear,
  Exit,
}

pub(super) struct RenderContext {
  keyword: String,
  escape_behavior: EscapeBehavior,
  repos: Vec<FoundRepo>,
  repo_selected_index: usize,
  cursor_show: bool,
}

#[derive(Debug)]
pub(super) enum ContextChange {
  RenderContextChanged,
  KeywordChanged,
  Finished(Result<Option<Repo>, MainModeError>),
}

#[derive(Clone)]
 struct MainModeWorker {
  context: Arc<RwLock<RenderContext>>,
  contextchange_tx: mpsc::Sender<ContextChange>,
}

async fn main_mode(finder: ReposFinder) -> Result<Option<Repo>, MainModeError> {
  let (contextchange_tx, mut contextchange_rx) = mpsc::channel::<ContextChange>(20);
  let (keywordchange_tx, keywordchange_rx) = mpsc::channel::<String>(20);

  let shared_context = Arc::new(RwLock::<RenderContext>::new(RenderContext {
    keyword: String::new(),
    escape_behavior: EscapeBehavior::Exit,
    repos: vec![],
    repo_selected_index: 0,
    cursor_show: false,
  }));

  let worker = MainModeWorker {
    context: shared_context.clone(),
    contextchange_tx,
  };

  let keyword_change_worker =
    tokio::spawn(keyword_change(finder, keywordchange_rx, worker.clone()));
  let key_input_worker = tokio::spawn(key_input(worker.clone()));
  let cursor_blinker_worker = tokio::spawn(cursor_blinker(worker.clone()));

  worker
    .contextchange_tx
    .send(ContextChange::RenderContextChanged)
    .await
    .unwrap();
  let result = loop {
    match contextchange_rx.recv().await.unwrap() {
      ContextChange::RenderContextChanged => {
        let context = shared_context.write().await;
        if let Err(e) = render(context).await {
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
  tokio::join!(
    keyword_change_worker,
    key_input_worker,
    cursor_blinker_worker
  )
  .0
  .map_err(|_| MainModeError::WorkerJoinError)?;
  result
}

async fn render(mut context: RwLockWriteGuard<'_, RenderContext>) -> Result<(), MainModeError> {
  let mut stderr = stderr();
  let (width, height) = terminal::size().map_err(|_| MainModeError::TerminalSizeUnavailable)?;
  let (width, height) = (width as i16, height as i16);

  queue!(
    stderr,
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
    stderr,
    cursor::MoveTo(0, 0),
    style::SetBackgroundColor(PINK_COLOR),
    style::SetForegroundColor(style::Color::White),
    style::SetAttribute(style::Attribute::Bold),
    style::Print(header_text),
    style::ResetColor,
  )
  .map_err(|_| MainModeError::StdoutWriteError)?;

  safe_move_to(
    &mut stderr,
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
    stderr,
    style::SetForegroundColor(style::Color::DarkGrey),
    style::Print(match context.escape_behavior {
      EscapeBehavior::Clear => CLEAR_MESSAGE,
      EscapeBehavior::Exit => EXIT_MESSAGE,
    }),
    style::ResetColor,
  )
  .map_err(|_| MainModeError::StdoutWriteError)?;

  safe_move_to(&mut stderr, 0, 1, width, height)?;
  let horizontal_line = safe_repeat(HORIZONTAL_LINE, width as isize - 2)?;
  queue!(
    stderr,
    style::SetForegroundColor(LIGHTER_PINK_COLOR),
    style::Print(TOP_LEFT_CORNER),
    style::Print(&horizontal_line),
    style::Print(TOP_RIGHT_CORNER),
    style::ResetColor,
  )
  .map_err(|_| MainModeError::StdoutWriteError)?;

  safe_move_to(&mut stderr, 0, 2, width, height)?;
  queue!(
    stderr,
    style::SetForegroundColor(LIGHTER_PINK_COLOR),
    style::Print(VERTICAL_LINE),
    style::ResetColor,
    style::Print(" 🔎 "),
    // style::SetAttribute(style::Attribute::Bold),
    style::Print(&context.keyword),
    style::ResetColor,
  )
  .map_err(|_| MainModeError::StdoutWriteError)?;

  safe_move_to(&mut stderr, width - 1, 2, width, height)?;
  queue!(
    stderr,
    style::SetForegroundColor(LIGHTER_PINK_COLOR),
    style::Print(VERTICAL_LINE),
    style::ResetColor,
  )
  .map_err(|_| MainModeError::StdoutWriteError)?;

  safe_move_to(&mut stderr, 0, 3, width, height)?;
  queue!(
    stderr,
    style::SetForegroundColor(LIGHTER_PINK_COLOR),
    style::Print(BOTTOM_LEFT_CORNER),
    style::Print(&horizontal_line),
    style::Print(BOTTOM_RIGHT_CORNER),
    style::ResetColor,
  )
  .map_err(|_| MainModeError::StdoutWriteError)?;

  let repo_views = height - 5;
  let rendering_repos = &context.repos[..cmp::min(repo_views as usize, context.repos.len())];
  let mut repo_selected_index = context.repo_selected_index;

  repo_selected_index = repo_selected_index.min(rendering_repos.len().max(1) - 1);

  rendering_repos.iter().enumerate().for_each(|(i, repo)| {
    safe_move_to(&mut stderr, 0, 5 + i as i16, width, height).unwrap();
    let (before, bold, after) = split_by_matched(&repo.repo.name, &repo.matched_string);
    if repo_selected_index == i {
      queue!(
        stderr,
        style::Print(" "),
        style::SetBackgroundColor(BACKGROUND_PINK_COLOR),
        style::SetForegroundColor(style::Color::White),
        style::Print(" » "),
        style::Print(before),
        style::SetAttribute(style::Attribute::Bold),
        style::Print(bold),
        style::SetAttribute(style::Attribute::Reset),
        style::SetBackgroundColor(BACKGROUND_PINK_COLOR),
        style::SetForegroundColor(style::Color::White),
        style::Print(after),
        style::Print("  "),
        style::ResetColor,
      )
      .unwrap();
    } else {
      queue!(
        stderr,
        style::Print(" "),
        style::SetForegroundColor(LIGHTER_PINK_COLOR),
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
    }
  });

  context.repo_selected_index = repo_selected_index;

  safe_move_to(
    &mut stderr,
    cmp::min(5 + context.keyword.len(), width as usize - 1) as i16,
    2,
    width,
    height,
  )?;

  if context.cursor_show {
    queue!(stderr, cursor::Show)
  } else {
    queue!(stderr, cursor::Hide)
  }
  .map_err(|_| MainModeError::StdoutWriteError)?;

  queue!(stderr, cursor::SetCursorStyle::SteadyUnderScore,)
    .map_err(|_| MainModeError::StdoutWriteError)?;

  stderr
    .flush()
    .map_err(|_| MainModeError::StdoutWriteError)?;

  Ok(())
}
