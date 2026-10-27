//! Export des règles en texte YAML portable, dans l'ordre d'évaluation.

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;
use crate::portability::envelope::wrap;
use crate::portability::rules::{StoredRefs, to_portable};

/// Écrit les règles avec des références en clair ; celle dont une cible a disparu est nommée et laissée de côté
pub(crate) fn handle_export_rules(engine: &Engine) -> Result<Response, MiraError> {
    let (items, skipped) = engine.database.read(|tx| {
        let refs = StoredRefs::new(tx);
        let (mut items, mut skipped) = (Vec::new(), Vec::new());
        for rule in tx.list_rules()? {
            match to_portable(&rule, &refs) {
                Ok(item) => items.push(item),
                Err(_) => skipped.push(rule.name),
            }
            if let Some(failure) = refs.take_failure() {
                return Err(failure);
            }
        }
        Ok((items, skipped))
    })?;
    let yaml = wrap("rules", "rules", items)?;
    Ok(Response::Rule(RuleResponse::RulesExport { yaml, skipped }))
}
