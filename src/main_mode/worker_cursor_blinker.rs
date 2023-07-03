use std::time::Duration;

use crate::main_mode::ContextChange;

use super::MainModeWorker;

pub(super) async fn cursor_blinker(
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
      _ = tokio::time::sleep(Duration::from_millis(500)) => {
        {
          let mut context = context.write().await;
          context.cursor_show = !context.cursor_show;
        }
        contextchange_tx.send(ContextChange::RenderContextChanged).await.unwrap();
      }
    }
  }
}
