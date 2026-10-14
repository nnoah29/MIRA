//! Désignation des messages visés par une fenêtre : objet et expéditeur, neutralisés.

use mira_assembly::mira_vocabulary::MessageSummary;

use crate::view::message_list::row::sender_text;
use crate::view::safe_text::display_safe;

/// « objet » de expéditeur, suivi du nombre d'autres messages s'il y en a.
pub fn target_label(messages: &[MessageSummary]) -> String {
    let Some(first) = messages.first() else {
        return String::new();
    };
    let label = format!(
        "« {} » de {}",
        display_safe(&first.subject),
        sender_text(first)
    );
    match messages.len() {
        0 | 1 => label,
        count => format!("{label} et {} autre(s)", count - 1),
    }
}
