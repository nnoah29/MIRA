//! Texte stocké pour le genre et l'origine d'un souvenir, dans les deux sens.

use mira_vocabulary::{MemoryKind, MemorySource};

/// Texte stocké pour un genre
pub(super) fn kind_text(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Fact => "fact",
        MemoryKind::Preference => "preference",
        MemoryKind::Action => "action",
    }
}

/// Texte stocké pour une origine
pub(super) fn source_text(source: MemorySource) -> &'static str {
    match source {
        MemorySource::User => "user",
        MemorySource::Ai => "ai",
    }
}

/// Genre d'un texte stocké ; texte inconnu : `None`
pub(super) fn parse_kind(text: &str) -> Option<MemoryKind> {
    match text {
        "fact" => Some(MemoryKind::Fact),
        "preference" => Some(MemoryKind::Preference),
        "action" => Some(MemoryKind::Action),
        _ => None,
    }
}

/// Origine d'un texte stocké ; texte inconnu : `None`
pub(super) fn parse_source(text: &str) -> Option<MemorySource> {
    match text {
        "user" => Some(MemorySource::User),
        "ai" => Some(MemorySource::Ai),
        _ => None,
    }
}
