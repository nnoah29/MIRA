//! Types autorisés, requêtes avec contexte et lecture du journal pour les tests de confidentialité.

use mira_core::api::ChatStart;
use mira_vocabulary::{AiConnection, AiJournalEntry, ContentKind, ContextPart, MaskingSettings};

use super::connection::chat_start;
use crate::ai_models_support::ModelsFixture;

/// Autorise exactement ces types de contenu pour cette connexion
pub fn allow_kinds(fixture: &ModelsFixture, connection: &AiConnection, kinds: &[ContentKind]) {
    fixture
        .db
        .write(|tx| tx.set_ai_allowed_kinds(&connection.id, kinds))
        .unwrap();
}

/// Impose ce réglage de masquage à cette connexion
pub fn set_masking(fixture: &ModelsFixture, connection: &AiConnection, masking: &MaskingSettings) {
    fixture
        .db
        .write(|tx| tx.set_ai_masking(&connection.id, masking))
        .unwrap();
}

/// Démarrage d'une discussion portant ces parties de contexte
pub fn chat_start_with(
    connection: &AiConnection,
    text: &str,
    parts: Vec<ContextPart>,
) -> ChatStart {
    let mut start = chat_start(connection, text);
    start.request.context = parts;
    start
}

/// Lignes du journal des envois, de la plus récente à la plus ancienne
pub fn journal_rows(fixture: &ModelsFixture) -> Vec<AiJournalEntry> {
    fixture
        .db
        .read(|tx| tx.list_ai_journal(5000, None))
        .unwrap()
}

/// Tout le texte des colonnes de la table du journal, relu directement dans la base
pub fn journal_dump(fixture: &ModelsFixture) -> String {
    let raw = rusqlite::Connection::open(&fixture.database_path).unwrap();
    let columns = "id||at||operation||connection||provider||model||kinds||outcome";
    let query = format!("SELECT group_concat({columns}) FROM ai_journal");
    raw.query_row(&query, [], |row| row.get(0)).unwrap()
}
