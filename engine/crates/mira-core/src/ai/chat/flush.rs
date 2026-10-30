//! Fin de restitution : le texte retenu sort toujours avant le `Done` final, même après échec ou annulation.

use super::ending::Ending;
use super::guard::{Guard, Interrupt};
use super::progress::Progress;

/// Remet ce que la restitution retenait ; un lecteur parti fait de la génération un abandon.
pub(super) async fn flush_pending(
    ending: Ending,
    guard: &Guard<'_>,
    progress: &mut Progress,
) -> Ending {
    if ending == Ending::Abandoned {
        return ending;
    }
    for delta in progress.answer.flush() {
        match guard.send(delta.clone()).await {
            Ok(()) => {}
            Err(Interrupt::Abandoned) => return Ending::Abandoned,
            Err(Interrupt::Cancelled) => guard.send_after_cancel(delta).await,
        }
    }
    ending
}
