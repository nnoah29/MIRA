//! Aide aux tests de rôle des dossiers.

use mira_core::mailboxes::infer_mailbox_role;
use mira_vocabulary::{MailboxRole, Provider, RawMailbox};

/// Déduit le rôle d'une boîte décrite par son nom, son séparateur et ses attributs
pub fn role(
    provider: &dyn Provider,
    name: &str,
    delimiter: Option<&str>,
    attrs: &[&str],
) -> MailboxRole {
    let raw = RawMailbox {
        name: name.to_string(),
        delimiter: delimiter.map(ToString::to_string),
        attributes: attrs.iter().map(ToString::to_string).collect(),
    };
    infer_mailbox_role(provider, &raw)
}
