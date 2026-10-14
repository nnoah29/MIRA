//! Connexion dédiée en IDLE sur la réception : elle écoute, le travail se fait sur une autre connexion.

use std::time::Duration;

use mira_vocabulary::MiraError;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::imap::IdleOutcome;
use crate::sync::session::Connection;

/// Ce que l'écoute IDLE signale à la tâche du compte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleSignal {
    Changed,
    Lost,
}

/// Écoute en cours sur la réception ; la connexion se ferme quand l'écoute est détruite.
#[derive(Debug)]
pub struct IdleWatcher {
    pub signals: mpsc::Receiver<IdleSignal>,
    task: JoinHandle<()>,
}

impl Drop for IdleWatcher {
    /// Arrête l'écoute, ce qui ferme la connexion et rend son permis.
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Boucle d'écoute : renouvelle l'IDLE au délai donné et signale chaque changement.
async fn listen(mut connection: Connection, renew: Duration, sender: mpsc::Sender<IdleSignal>) {
    loop {
        let signal = match connection.client.idle_until_change(renew).await {
            Ok(IdleOutcome::Changed) => IdleSignal::Changed,
            Ok(IdleOutcome::TimedOut) => continue,
            Err(_) => IdleSignal::Lost,
        };
        if signal == IdleSignal::Changed {
            let _ = sender.try_send(IdleSignal::Changed);
            continue;
        }
        let _ = sender.send(IdleSignal::Lost).await;
        return;
    }
}

/// Sélectionne la réception sur la connexion donnée puis lance l'écoute IDLE en arrière-plan.
pub async fn start_idle_watcher(
    mut connection: Connection,
    inbox_name: &str,
    renew: Duration,
) -> Result<IdleWatcher, MiraError> {
    connection.client.select(inbox_name, None).await?;
    let (sender, signals) = mpsc::channel(1);
    let task = tokio::spawn(listen(connection, renew, sender));
    Ok(IdleWatcher { signals, task })
}
