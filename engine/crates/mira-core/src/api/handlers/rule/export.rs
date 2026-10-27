//! Export des règles en texte YAML portable, dans l'ordre d'évaluation.

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;
use crate::portability::envelope::wrap;
use crate::portability::rules::{StoredRefs, to_portable};

/// Écrit toutes les règles avec des références en clair ; une cible disparue fait échouer l'export
pub(crate) fn handle_export_rules(engine: &Engine) -> Result<Response, MiraError> {
    let items = engine.database.read(|tx| {
        let refs = StoredRefs::new(tx);
        let mut items = Vec::new();
        for rule in tx.list_rules()? {
            items.push(to_portable(&rule, &refs));
            if let Some(failure) = refs.take_failure() {
                return Err(failure);
            }
        }
        items.into_iter().collect::<Result<Vec<_>, _>>()
    })?;
    let yaml = wrap("rules", "rules", items)?;
    Ok(Response::Rule(RuleResponse::RulesExport { yaml }))
}
