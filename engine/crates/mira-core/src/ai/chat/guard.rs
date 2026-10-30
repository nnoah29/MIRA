//! Attentes interruptibles de la tâche : chaque attente surveille l'annulation et le départ du lecteur.

use std::future::Future;
use std::time::Duration;

use mira_vocabulary::ChatDelta;
use tokio::sync::mpsc;
use tokio::time::{Instant, interval_at, timeout};

use crate::operations::OperationHandle;

const CANCEL_CHECK: Duration = Duration::from_millis(100);
const CANCEL_GRACE: Duration = Duration::from_secs(2);

/// Raison pour laquelle une attente a été abandonnée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Interrupt {
    Cancelled,
    Abandoned,
}

/// Surveille l'opération et le canal du lecteur pendant les attentes de la tâche.
pub(super) struct Guard<'a> {
    handle: &'a OperationHandle,
    deltas: &'a mpsc::Sender<ChatDelta>,
}

impl<'a> Guard<'a> {
    /// Associe la poignée de l'opération et le canal vers le lecteur.
    pub(super) fn new(handle: &'a OperationHandle, deltas: &'a mpsc::Sender<ChatDelta>) -> Self {
        Self { handle, deltas }
    }

    /// Attend le résultat, sauf annulation (testée toutes les 100 ms) ou lecteur parti.
    pub(super) async fn wait<F: Future>(&self, future: F) -> Result<F::Output, Interrupt> {
        let mut ticker = interval_at(Instant::now() + CANCEL_CHECK, CANCEL_CHECK);
        tokio::pin!(future);
        loop {
            if self.handle.is_cancelled() {
                return Err(Interrupt::Cancelled);
            }
            tokio::select! {
                output = &mut future => return Ok(output),
                () = self.deltas.closed() => return Err(Interrupt::Abandoned),
                _ = ticker.tick() => {}
            }
        }
    }

    /// Transmet un delta au lecteur, même canal plein, sans jamais bloquer une annulation.
    pub(super) async fn send(&self, delta: ChatDelta) -> Result<(), Interrupt> {
        match self.wait(self.deltas.send(delta)).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) => Err(Interrupt::Abandoned),
            Err(interrupt) => Err(interrupt),
        }
    }

    /// Après une annulation, tente encore de remettre le delta, sans attendre un lecteur qui ne lit plus.
    pub(super) async fn send_after_cancel(&self, delta: ChatDelta) {
        let _ = timeout(CANCEL_GRACE, self.deltas.send(delta)).await;
    }
}
