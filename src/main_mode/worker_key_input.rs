use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

use crate::main_mode::{ContextChange, EscapeBehavior, MainModeError};

use super::MainModeWorker;

pub(super) async fn key_input(
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
                code: KeyCode::Up,
                ..
              } => {
                {
                  let mut context = context.write().await;
                  if context.repo_selected_index <= 0 { continue; }
                  context.repo_selected_index -= 1;
                }
                contextchange_tx.send(ContextChange::RenderContextChanged).await.unwrap();
              }
              KeyEvent {
                code: KeyCode::Down,
                ..
              } => {
                {
                  let mut context = context.write().await;
                  context.repo_selected_index += 1;
                }
                contextchange_tx.send(ContextChange::RenderContextChanged).await.unwrap();
              }
              KeyEvent {
                code: KeyCode::Backspace,
                ..
              } => {
                {
                  let mut context = context.write().await;
                  context.keyword.pop();
                }
                contextchange_tx.send(ContextChange::KeywordChanged).await.unwrap();
              }
              KeyEvent {
                code: KeyCode::Char(c),
                ..
              } => {
                {
                  let mut context = context.write().await;
                  context.keyword.push(c);
                }
                contextchange_tx.send(ContextChange::KeywordChanged).await.unwrap();
              }
              _ => {}
            }
          }
          Ok(Ok(_)) => {
            contextchange_tx.send(ContextChange::RenderContextChanged).await.unwrap();
          }
          Ok(Err(_)) | Err(_) => {
            contextchange_tx.send(ContextChange::Finished(Err(MainModeError::EventReadError))).await.unwrap();
            break;
          }
        }
      }
    }
  }
}
