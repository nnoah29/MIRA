//! Rappel automatique : les souvenirs proches du dernier message rejoignent le contexte de l'assistant, jamais au prix d'une panne.

use mira_vocabulary::{
    AiConnection, ChatMode, ChatRole, ContentKind, ContextPart, MaskingSettings,
};

use crate::ai::masking::Masker;
use crate::ai::tools::render_hits;
use crate::api::{ChatStart, Engine};

const MAX_RECALLED: usize = 5;
const MAX_QUERY_CHARS: usize = 1_000;
const LABEL: &str = "Souvenirs de l'utilisateur";

/// Dit si les souvenirs peuvent être envoyés : sur l'appareil, ou si la connexion autorise le type « mémoire ».
pub fn memory_allowed(allowed: &[ContentKind], local: bool) -> bool {
    local || allowed.contains(&ContentKind::Memory)
}

/// Dernier message de l'utilisateur, masqué comme pour le service d'IA puis borné ; vide s'il n'y en a pas.
fn masked_query(start: &ChatStart) -> String {
    let last = start
        .request
        .messages
        .iter()
        .rev()
        .find(|message| message.role == ChatRole::User);
    let text = last.map(|message| message.text.trim()).unwrap_or_default();
    let mut masker = Masker::new(&MaskingSettings::remote_default());
    masker.reserve(text);
    masker.mask(text).chars().take(MAX_QUERY_CHARS).collect()
}

/// Dit si le rappel a lieu : mémoire activée, type autorisé et mode assistant.
fn recall_wanted(
    engine: &Engine,
    start: &ChatStart,
    connection: &AiConnection,
    local: bool,
) -> bool {
    let enabled = engine
        .database
        .read(|tx| tx.ai_memory_settings())
        .is_ok_and(|settings| settings.enabled);
    let allowed = memory_allowed(&connection.allowed_kinds, local);
    enabled && allowed && matches!(start.mode, ChatMode::Assistant)
}

/// Ajoute au contexte les souvenirs proches du dernier message ; mémoire désactivée, mode composeur, type non autorisé ou panne : rien.
pub(super) async fn add_memories(
    engine: &Engine,
    start: &mut ChatStart,
    connection: &AiConnection,
    local: bool,
) {
    let query = masked_query(start);
    if query.is_empty() || !recall_wanted(engine, start, connection, local) {
        return;
    }
    let Ok(store) = engine.memory_store().await else {
        return;
    };
    let Ok(hits) = store.recall(&query, MAX_RECALLED).await else {
        return;
    };
    if !hits.is_empty() {
        start.request.context.push(ContextPart {
            kind: ContentKind::Memory,
            label: LABEL.to_string(),
            text: render_hits(&hits),
        });
    }
}
