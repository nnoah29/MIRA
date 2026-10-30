//! Génération en flux exposée par la façade (comme `subscribe`) : un identifiant d'opération et des deltas.

use std::sync::{Arc, Weak};

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
    #[serde(default)]
    pub allow_tools: bool,
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
    /// Vérifie localement la demande, ouvre la ligne de journal, puis lance la génération en tâche de fond, sans outils.
    pub async fn chat_stream(&self, start: ChatStart) -> Result<ChatStream, MiraError> {
        self.begin_chat(start, None)
    }

    /// Comme `chat_stream`, avec les outils en lecture seule quand `allow_tools` est vrai et que le service les supporte.
    pub async fn chat_stream_with_tools(
        self: &Arc<Self>,
        start: ChatStart,
    ) -> Result<ChatStream, MiraError> {
        self.begin_chat(start, Some(Arc::downgrade(self)))
    }

    /// Prépare le plan, ouvre la ligne de journal et lance la tâche ; `host` donne à la tâche l'accès aux lectures.
    fn begin_chat(
        &self,
        start: ChatStart,
        host: Option<Weak<Engine>>,
    ) -> Result<ChatStream, MiraError> {
        let plan = prepare(self, start, host)?;
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
