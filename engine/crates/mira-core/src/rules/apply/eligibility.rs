//! Boîtes dans lesquelles l'arrivée d'un message déclenche les règles.

use mira_vocabulary::MailboxRole;

/// Faux pour Envoyés, Brouillons, Corbeille et Spam : les règles n'y sont jamais évaluées
pub fn rules_apply_in(role: MailboxRole) -> bool {
    !matches!(
        role,
        MailboxRole::Sent | MailboxRole::Drafts | MailboxRole::Trash | MailboxRole::Junk
    )
}
