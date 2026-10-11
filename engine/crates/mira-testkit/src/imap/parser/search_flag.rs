//! Critères de recherche fondés sur les drapeaux système.

use super::search::Criterion;

const FLAG_KEYS: [(&str, &str, bool); 10] = [
    ("SEEN", "\\Seen", true),
    ("UNSEEN", "\\Seen", false),
    ("ANSWERED", "\\Answered", true),
    ("UNANSWERED", "\\Answered", false),
    ("FLAGGED", "\\Flagged", true),
    ("UNFLAGGED", "\\Flagged", false),
    ("DELETED", "\\Deleted", true),
    ("UNDELETED", "\\Deleted", false),
    ("DRAFT", "\\Draft", true),
    ("UNDRAFT", "\\Draft", false),
];

/// Critère de drapeau correspondant à un mot-clé comme SEEN ou UNFLAGGED
pub(crate) fn flag_criterion(word: &str) -> Option<Criterion> {
    FLAG_KEYS
        .iter()
        .find(|(key, _, _)| *key == word)
        .map(|(_, flag, present)| Criterion::Flag((*flag).to_string(), *present))
}
