//! Citation des arguments de commande : chaînes entre guillemets échappées, ensembles d'UID.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::mailbox_name::encode_mailbox_name;

const UIDS_PER_COMMAND: usize = 500;

/// Refuse un texte qui contient un saut de ligne ou un octet nul.
pub(crate) fn reject_control_characters(text: &str) -> Result<(), MiraError> {
    if text.chars().any(|c| matches!(c, '\r' | '\n' | '\0')) {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "argument IMAP contenant un caractère de contrôle",
        ));
    }
    Ok(())
}

/// Cite un nom de boîte lisible : encodé en UTF-7 modifié puis mis entre guillemets.
pub(crate) fn quote_mailbox(name: &str) -> Result<String, MiraError> {
    reject_control_characters(name)?;
    quote(&encode_mailbox_name(name))
}

/// Met un texte entre guillemets en échappant `"` et `\` ; refuse les sauts de ligne.
pub(crate) fn quote(text: &str) -> Result<String, MiraError> {
    reject_control_characters(text)?;
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    Ok(format!("\"{escaped}\""))
}

/// Écrit des UID triés et dédoublonnés en ensemble IMAP (`1:3,7`).
pub(crate) fn format_uid_set(uids: &[u32]) -> String {
    let mut sorted = uids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    let mut parts: Vec<String> = Vec::new();
    let mut index = 0;
    while index < sorted.len() {
        let start = sorted[index];
        let mut end = start;
        while index + 1 < sorted.len() && sorted[index + 1] == end.saturating_add(1) {
            index += 1;
            end = sorted[index];
        }
        parts.push(if start == end {
            start.to_string()
        } else {
            format!("{start}:{end}")
        });
        index += 1;
    }
    parts.join(",")
}

/// Découpe les UID en ensembles de taille bornée, un par commande.
pub(crate) fn uid_sets(uids: &[u32]) -> Vec<String> {
    let mut sorted = uids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
        .chunks(UIDS_PER_COMMAND)
        .map(format_uid_set)
        .collect()
}
