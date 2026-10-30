//! Moteur partagé (`Arc`) ouvert sur un faux serveur d'IA et une boîte garnie, pour la boucle d'outils.
#![allow(dead_code)]

use std::sync::Arc;

use mira_core::api::{ChatStart, ChatStream, Engine};
use mira_core::storage::Database;
use mira_testkit::ai::FakeAiServer;
use mira_vocabulary::{AiConnection, ChatMessage, ChatRequest, ChatRole, WireFormat};
use tempfile::TempDir;

use super::seed::{local_connection, seed};
use crate::ai_models_support::{ModelsFixture, TestProvider};

/// Ce qu'un test de la boucle manipule.
pub struct LoopFixture {
    pub engine: Arc<Engine>,
    pub db: Database,
    pub server: FakeAiServer,
    pub connection: AiConnection,
    pub home: TempDir,
}

impl LoopFixture {
    /// Fournisseur de ce format qui annonce les outils, connexion locale `conn-1`, boîte garnie.
    pub async fn start(format: WireFormat) -> Self {
        Self::start_with(format, true).await
    }

    /// Comme `start`, en choisissant si le fournisseur supporte les outils.
    pub async fn start_with(format: WireFormat, supports_tools: bool) -> Self {
        let mut provider = TestProvider::new("test-provider");
        provider.format = format;
        provider.supports_tools = supports_tools;
        let models = ModelsFixture::start(provider).await;
        let connection = local_connection(&models);
        models
            .db
            .write(|tx| tx.insert_ai_connection(&connection, 1))
            .unwrap();
        seed(&models.db);
        let ModelsFixture {
            home,
            engine,
            db,
            server,
            ..
        } = models;
        let engine = Arc::new(engine);
        Self {
            engine,
            db,
            server,
            connection,
            home,
        }
    }

    /// Démarrage d'une discussion d'un seul message ; les outils sont permis si `allow_tools`.
    pub fn start_of(&self, text: &str, allow_tools: bool) -> ChatStart {
        let message = ChatMessage::plain(ChatRole::User, text.to_string());
        ChatStart {
            connection: Some(self.connection.id.clone()),
            request: ChatRequest {
                model: None,
                system: None,
                messages: vec![message],
                max_output_tokens: None,
                temperature: None,
                tools: Vec::new(),
                context: Vec::new(),
            },
            allow_tools,
        }
    }

    /// Pose la question avec les outils permis.
    pub async fn ask(&self, text: &str) -> ChatStream {
        let start = self.start_of(text, true);
        self.engine.chat_stream_with_tools(start).await.unwrap()
    }
}
