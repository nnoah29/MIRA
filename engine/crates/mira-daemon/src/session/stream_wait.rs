//! Attentes d'un flux : un crédit, ou l'arrêt, l'arrêt l'emportant toujours.

use super::stop::Stop;
use super::stream::Stream;

impl Stream {
    /// Consomme un crédit ; attend qu'il y en ait un, ou rend la raison de l'arrêt (prioritaire, même s'il reste des crédits).
    pub async fn take_credit(&self) -> Result<(), Stop> {
        tokio::select! {
            biased;
            why = self.stopped() => Err(why),
            permit = self.credits.acquire() => {
                permit.map(|permit| permit.forget()).map_err(|_| Stop::Cancelled)
            }
        }
    }

    /// Attend l'arrêt du flux et rend sa raison.
    pub async fn stopped(&self) -> Stop {
        let mut watcher = self.stop.subscribe();
        loop {
            if let Some(why) = *watcher.borrow_and_update() {
                return why;
            }
            if watcher.changed().await.is_err() {
                return Stop::Cancelled;
            }
        }
    }
}
