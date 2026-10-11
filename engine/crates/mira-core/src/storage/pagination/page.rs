//! Construction des pages : borne de reprise lue du curseur, découpe des lignes lues.

use mira_vocabulary::{MiraError, Page, PageRequest};

use super::cursor::{decode_cursor, encode_cursor};

/// Position de reprise (clé de tri, identifiant) tirée de la demande, si elle a un curseur
pub(crate) fn cursor_bounds(
    request: &PageRequest,
) -> Result<(Option<i64>, Option<String>), MiraError> {
    match &request.cursor {
        Some(cursor) => decode_cursor(cursor).map(|(key, id)| (Some(key), Some(id))),
        None => Ok((None, None)),
    }
}

/// Coupe les lignes lues (limite + 1 au plus) en une page et son curseur de suite
pub(crate) fn finish_page<T>(mut rows: Vec<(i64, String, T)>, limit: u32) -> Page<T> {
    let limit = usize::try_from(limit).unwrap_or(usize::MAX);
    let mut next_cursor = None;
    if rows.len() > limit {
        rows.truncate(limit);
        next_cursor = rows.last().map(|(key, id, _)| encode_cursor(*key, id));
    }
    Page {
        items: rows.into_iter().map(|(_, _, item)| item).collect(),
        next_cursor,
    }
}
