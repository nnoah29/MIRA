//! Déduction du rôle d'une boîte : SPECIAL-USE d'abord, puis INBOX, l'indice du provider, les noms usuels.

use mira_vocabulary::{MailboxRole, Provider, RawMailbox};

use super::common_names::role_from_common_name;
use super::special_use::role_from_special_use;

/// Déduit le rôle d'une boîte ; l'attribut SPECIAL-USE du serveur gagne toujours (10.14).
pub fn infer_mailbox_role(provider: &dyn Provider, raw: &RawMailbox) -> MailboxRole {
    if let Some(role) = role_from_special_use(&raw.attributes) {
        return role;
    }
    if raw.name.eq_ignore_ascii_case("INBOX") {
        return MailboxRole::Inbox;
    }
    if let Some(role) = provider.mailbox_role_hint(raw) {
        return role;
    }
    role_from_common_name(&raw.name, raw.delimiter.as_deref()).unwrap_or(MailboxRole::Other)
}
