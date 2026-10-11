//! Noms complets des boîtes créées ou renommées, d'après leur parent et le séparateur de hiérarchie.

use mira_vocabulary::Mailbox;

const DEFAULT_DELIMITER: &str = "/";

/// Séparateur de hiérarchie de la boîte, ou le séparateur par défaut
pub(super) fn delimiter_of(mailbox: &Mailbox) -> String {
    mailbox
        .delimiter
        .clone()
        .unwrap_or_else(|| DEFAULT_DELIMITER.to_string())
}

/// Nom complet d'une nouvelle boîte, placée sous son parent s'il y en a un
pub(super) fn child_name(parent: Option<&Mailbox>, leaf: &str) -> String {
    match parent {
        Some(parent) => format!("{}{}{leaf}", parent.name, delimiter_of(parent)),
        None => leaf.to_string(),
    }
}

/// Nom complet d'une boîte renommée : son chemin parent est conservé, seul le dernier segment change
pub(super) fn renamed_name(mailbox: &Mailbox, leaf: &str) -> String {
    let delimiter = delimiter_of(mailbox);
    match mailbox.name.rfind(&delimiter) {
        Some(index) => {
            let parent_path = mailbox.name.get(..index + delimiter.len()).unwrap_or("");
            format!("{parent_path}{leaf}")
        }
        None => leaf.to_string(),
    }
}
