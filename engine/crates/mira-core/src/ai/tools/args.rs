//! Arguments communs aux outils de messages : lecture JSON, filtres et bornes de dates.

use mira_vocabulary::{Cursor, PageRequest, Timestamp};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::ai::wire::rfc3339::unix_seconds;
use crate::reading::MessageFilters;

pub(super) const PAGE_SIZE: u32 = 20;

/// Filtres et reprise de page communs à la recherche et à la liste.
#[derive(Debug, Default, Deserialize)]
pub(super) struct CommonArgs {
    #[serde(default)]
    pub(super) unread_only: bool,
    #[serde(default)]
    pub(super) flagged_only: bool,
    #[serde(default)]
    pub(super) has_attachments: bool,
    pub(super) from_contains: Option<String>,
    pub(super) since: Option<String>,
    pub(super) until: Option<String>,
    pub(super) cursor: Option<String>,
}

/// Lit les arguments JSON de l'appel ; un texte illisible est expliqué au modèle.
pub(super) fn parse<T: DeserializeOwned>(arguments: &str) -> Result<T, String> {
    serde_json::from_str(arguments).map_err(|_| "arguments illisibles : JSON attendu".to_string())
}

/// Filtres du contrat tirés des arguments ; une date illisible est une erreur.
pub(super) fn filters_of(args: &CommonArgs) -> Result<MessageFilters, String> {
    Ok(MessageFilters {
        unread_only: args.unread_only,
        flagged_only: args.flagged_only,
        has_attachments: args.has_attachments,
        from_contains: args.from_contains.clone().filter(|text| !text.is_empty()),
        since: args
            .since
            .as_deref()
            .map(|text| instant(text, false))
            .transpose()?,
        until: args
            .until
            .as_deref()
            .map(|text| instant(text, true))
            .transpose()?,
    })
}

/// Instant d'une date RFC 3339, ou d'une date seule (début de journée, fin si `end_of_day`).
fn instant(text: &str, end_of_day: bool) -> Result<Timestamp, String> {
    let full = match text.len() {
        10 if end_of_day => format!("{text}T23:59:59Z"),
        10 => format!("{text}T00:00:00Z"),
        _ => text.to_string(),
    };
    let seconds = unix_seconds(&full)
        .ok_or_else(|| format!("date illisible « {text} » : AAAA-MM-JJ ou RFC 3339 attendu"))?;
    Ok(Timestamp::from_millis(seconds.saturating_mul(1000)))
}

/// Page de vingt éléments, reprise au curseur donné.
pub(super) fn page_of(cursor: Option<&String>) -> PageRequest {
    PageRequest {
        cursor: cursor.cloned().map(Cursor),
        limit: PAGE_SIZE,
    }
}
