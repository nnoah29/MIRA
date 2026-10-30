//! Génération en flux exposée par la façade (comme `subscribe`) : un identifiant d'opération et des deltas.

use std::fmt;
use std::sync::Arc;

use mira_vocabulary::{AiConnectionId, ChatDelta, ChatRequest, MiraError, OperationId};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use super::Engine;
use crate::ai::chat::ChatPlan;
use crate::ai::chat::{CHANNEL_CAPACITY, CHAT_OPERATION_KIND, ChatTask, open_row, prepare, run};
use crate::operations::OperationHandle;

/// Demande de génération ; sans connexion désignée, la connexion par défaut sert.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatStart {
    pub connection: Option<AiConnectionId>,
    pub request: ChatRequest,
}

/// Génération en cours : son opération (annulable) et les deltas, terminés par un seul `Done`.
pub struct ChatStream {
    pub operation: OperationId,
    deltas: mpsc::Receiver<ChatDelta>,
    done: Option<oneshot::Receiver<ChatDelta>>,
}

impl ChatStream {
    /// Rend le delta suivant ; le dernier est `Done`, puis plus rien.
    pub async fn next(&mut self) -> Option<ChatDelta> {
        if let Some(delta) = self.deltas.recv().await {
            return Some(delta);
        }
        self.done.take()?.await.ok()
    }
}

impl Engine {
    /// Vérifie localement la demande, ouvre la ligne de journal, puis lance la génération en tâche de fond.
    pub async fn chat_stream(&self, start: ChatStart) -> Result<ChatStream, MiraError> {
        let plan = prepare(self, start)?;
        let handle = self.operations.start(CHAT_OPERATION_KIND);
        if let Err(error) = open_row(self, &plan, handle.id()) {
            handle.complete(Err(&error));
            return Err(error);
        }
        Ok(self.spawn_chat(plan, handle))
    }

    /// Lance la tâche de génération et rend le flux de ses deltas.
    fn spawn_chat(&self, plan: ChatPlan, handle: OperationHandle) -> ChatStream {
        let operation = handle.id().clone();
        let (deltas_sender, deltas) = mpsc::channel(CHANNEL_CAPACITY);
        let (done_sender, done) = oneshot::channel();
        tokio::spawn(run(ChatTask {
            plan,
            handle,
            operations: self.operations.clone(),
            events: self.events.clone(),
            database: Arc::clone(&self.database),
            clock: Arc::clone(&self.config.clock),
            deltas: deltas_sender,
            done: done_sender,
        }));
        ChatStream {
            operation,
            deltas,
            done: Some(done),
        }
    }
}

impl fmt::Debug for ChatStart {
    /// Montre la connexion, le modèle et le nombre de messages, jamais leur texte ni la consigne.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatStart")
            .field("connection", &self.connection)
            .field("model", &self.request.model)
            .field("has_system", &self.request.system.is_some())
            .field("messages", &self.request.messages.len())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for ChatStream {
    /// Montre seulement l'identifiant de l'opération.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChatStream")
            .field("operation", &self.operation)
            .finish_non_exhaustive()
    }
}
