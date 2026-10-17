//! Requêtes du moteur exécutées dans des tâches ; chaque requête rend toujours une réponse, même si sa tâche panique.

use std::collections::HashMap;
use std::future::Future;
use std::time::Duration;

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::MiraError;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::{Id, JoinError, JoinHandle, JoinSet};

use super::task_failure::task_failed;
use crate::commands::Tag;
use crate::update::{Msg, Reply};

const DRAIN_DEADLINE: Duration = Duration::from_secs(3);

/// Tâches en vol, l'étiquette de chacune et la file où leurs réponses arrivent.
#[derive(Debug)]
pub struct RequestTasks {
    tasks: JoinSet<()>,
    tags: HashMap<Id, Tag>,
    sender: UnboundedSender<Msg>,
}

impl RequestTasks {
    /// Ensemble vide dont les réponses partent dans cette file.
    pub fn new(sender: UnboundedSender<Msg>) -> RequestTasks {
        RequestTasks {
            tasks: JoinSet::new(),
            tags: HashMap::new(),
            sender,
        }
    }

    /// Lance le travail ; sa réponse part étiquetée, et son étiquette est gardée au cas où il paniquerait.
    pub fn spawn<F>(&mut self, tag: Tag, work: F)
    where
        F: Future<Output = Result<Response, MiraError>> + Send + 'static,
    {
        let sender = self.sender.clone();
        let reply_tag = tag.clone();
        let handle = self.tasks.spawn(async move {
            let result = work.await;
            let _ = sender.send(Msg::Reply(Reply {
                tag: reply_tag,
                result,
            }));
        });
        self.tags.insert(handle.id(), tag);
    }

    /// Attend la fin d'une tâche et traite son issue ; `None` s'il n'y en a aucune en vol.
    pub async fn settle_next(&mut self) -> Option<()> {
        let joined = self.tasks.join_next_with_id().await?;
        self.settle(joined);
        Some(())
    }

    /// Oublie l'étiquette d'une tâche finie ; une panique devient une réponse d'erreur interne.
    fn settle(&mut self, joined: Result<(Id, ()), JoinError>) {
        match joined {
            Ok((id, ())) => {
                self.tags.remove(&id);
            }
            Err(error) => {
                let tag = self.tags.remove(&error.id());
                if let Some(tag) = tag.filter(|_| error.is_panic()) {
                    let reply = Reply {
                        tag,
                        result: Err(task_failed()),
                    };
                    let _ = self.sender.send(Msg::Reply(reply));
                }
            }
        }
    }

    /// Laisse finir les tâches en vol (délai borné) puis interrompt les dernières : plus aucun clone du moteur.
    pub async fn stop(mut self) {
        let drain = async { while self.tasks.join_next().await.is_some() {} };
        let _ = tokio::time::timeout(DRAIN_DEADLINE, drain).await;
        self.tasks.shutdown().await;
    }

    /// Même arrêt dans une tâche à part : rend la main tout de suite, la poignée dit quand c'est fini.
    pub fn stop_detached(self) -> JoinHandle<()> {
        tokio::spawn(self.stop())
    }
}
