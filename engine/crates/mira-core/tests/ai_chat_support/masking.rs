//! Connexion au masquage imposé, lancement d'une génération et texte de la réponse reçue.

use mira_core::api::ChatStart;
use mira_vocabulary::{AiConnection, ChatDelta, MaskingSettings};

use super::connection::add_chat_connection;
use super::flow::collect;
use super::privacy::set_masking;
use crate::ai_models_support::ModelsFixture;

/// Connexion locale `conn-1` dont le masquage est imposé à ce réglage
pub fn connection_with(fixture: &ModelsFixture, masking: &MaskingSettings) -> AiConnection {
    let base = Some(fixture.server.base_url());
    let connection = add_chat_connection(fixture, "conn-1", base, Some("model-a"));
    set_masking(fixture, &connection, masking);
    connection
}

/// Texte de la réponse reçue par le client, deltas de texte mis bout à bout
pub fn answer_of(deltas: &[ChatDelta]) -> String {
    let texts = deltas.iter().filter_map(|delta| match delta {
        ChatDelta::Text { text } => Some(text.as_str()),
        _ => None,
    });
    texts.collect()
}

/// Lance la génération et rend tous les deltas reçus
pub async fn run_chat(fixture: &ModelsFixture, start: ChatStart) -> Vec<ChatDelta> {
    let mut stream = fixture.engine.chat_stream(start).await.unwrap();
    collect(&mut stream).await
}
