//! Import d'une règle du fichier : références résolues, doublon traité, contrôle puis écriture.

use std::collections::HashSet;

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{
    ErrorCategory, MiraError, Rule, RuleId, RuleImportOutcome, RuleImportRefusal,
};
use serde_json::Value;

use super::writing::{check_writable, with_normalized_scope};
use crate::portability::rules::{PortableError, StoredRefs, from_portable};
use crate::storage::Tx;

/// Sort d'une règle, et son identifiant quand elle a été écrite
type Imported = (RuleImportOutcome, Option<RuleId>);

/// Refus portant cette raison
fn refused(reason: RuleImportRefusal) -> RuleImportOutcome {
    RuleImportOutcome::Refused { reason }
}

/// Raison de refus d'une règle du fichier dont la conversion a échoué
fn refusal_of(error: PortableError) -> RuleImportRefusal {
    match error {
        PortableError::UnknownAccount => RuleImportRefusal::UnknownAccount,
        PortableError::UnknownMailbox => RuleImportRefusal::UnknownMailbox,
        PortableError::UnknownLabel => RuleImportRefusal::UnknownLabel,
        PortableError::UnknownTemplate => RuleImportRefusal::UnknownTemplate,
        PortableError::Invalid => RuleImportRefusal::InvalidRule {
            code: invalid_input::PARAMETER.0.to_string(),
        },
    }
}

/// Convertit puis importe une règle du fichier ; une panne de stockage interrompt tout l'import
pub(super) fn import_item(
    tx: &Tx<'_>,
    refs: &StoredRefs<'_, '_>,
    value: Value,
    replace: bool,
    seen: &mut HashSet<String>,
) -> Result<Imported, MiraError> {
    let parsed = from_portable(value, refs);
    if let Some(failure) = refs.take_failure() {
        return Err(failure);
    }
    match parsed {
        Ok(rule) => write_rule(tx, with_normalized_scope(rule), replace, seen),
        Err(error) => Ok((refused(refusal_of(error)), None)),
    }
}

/// Écrit la règle : doublon interne ou existant, contrôle, puis insertion ou remplacement
fn write_rule(
    tx: &Tx<'_>,
    rule: Rule,
    replace: bool,
    seen: &mut HashSet<String>,
) -> Result<Imported, MiraError> {
    if !seen.insert(rule.id.as_str().to_string()) {
        let code = invalid_input::RULE_DUPLICATE_IN_FILE.0.to_string();
        return Ok((refused(RuleImportRefusal::InvalidRule { code }), None));
    }
    let exists = tx.get_rule(&rule.id)?.is_some();
    if exists && !replace {
        return Ok((RuleImportOutcome::AlreadyPresent, None));
    }
    if let Some(refusal) = check(tx, &rule)? {
        return Ok((refusal, None));
    }
    let outcome = if exists {
        tx.update_rule(&rule)?;
        RuleImportOutcome::Replaced
    } else {
        tx.insert_rule(&rule)?;
        RuleImportOutcome::Imported
    };
    Ok((outcome, Some(rule.id)))
}

/// Refus quand le contrôle échoue à cause de la règle, erreur quand c'est le stockage
fn check(tx: &Tx<'_>, rule: &Rule) -> Result<Option<RuleImportOutcome>, MiraError> {
    match check_writable(tx, rule) {
        Ok(_) => Ok(None),
        Err(error)
            if matches!(
                error.category,
                ErrorCategory::InvalidInput | ErrorCategory::NotFound
            ) =>
        {
            Ok(Some(refused(RuleImportRefusal::InvalidRule {
                code: error.code,
            })))
        }
        Err(error) => Err(error),
    }
}
