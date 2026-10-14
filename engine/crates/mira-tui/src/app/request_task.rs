//! Exécution d'une requête du moteur dans une tâche, sa réponse renvoyée à la boucle.

use std::sync::Arc;

use mira_assembly::api::{Engine, Request};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::JoinSet;

use crate::commands::Tag;
use crate::update::{Msg, Reply};

/// Lance la requête ; la tâche garde un clone du moteur jusqu'à sa fin, puis envoie `Msg::Reply`.
pub fn spawn_request(
    tasks: &mut JoinSet<()>,
    engine: &Arc<Engine>,
    sender: &UnboundedSender<Msg>,
    tag: Tag,
    request: Request,
) {
    let engine = Arc::clone(engine);
    let sender = sender.clone();
    tasks.spawn(async move {
        let result = engine.execute(request).await;
        let _ = sender.send(Msg::Reply(Reply { tag, result }));
    });
}
