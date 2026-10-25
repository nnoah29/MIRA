//! Aides communes aux tests du dépôt des modèles de messages.

use mira_core::storage::Database;
use mira_vocabulary::{MessageTemplate, MiraError, TemplateId};

/// Modèle de test portant ce nom
pub fn template(id: &str, name: &str) -> MessageTemplate {
    MessageTemplate {
        id: TemplateId::from_raw(id),
        name: name.to_string(),
        subject: Some("Objet".to_string()),
        body: "Bonjour {{contact.nom}}".to_string(),
        created_at: 1,
        updated_at: 1,
    }
}

/// Insère le modèle dans une base fraîche déjà ouverte
pub fn insert(db: &Database, item: &MessageTemplate) -> Result<(), MiraError> {
    db.write(|tx| tx.insert_template(item))
}

/// Code de l'erreur rendue
pub fn code_of(result: Result<(), MiraError>) -> String {
    result.unwrap_err().code
}
