use tokio::sync::mpsc::Receiver;

use crate::{
  finder::ReposFinder,
  main_mode::{ContextChange, EscapeBehavior},
};

use super::MainModeWorker;

pub(super) async fn keyword_change(
  finder: ReposFinder,
  mut keywordchange_rx: Receiver<String>,
  MainModeWorker {
    context,
    contextchange_tx,
  }: MainModeWorker,
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
