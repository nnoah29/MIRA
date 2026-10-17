//! Traduction des filtres en conditions SQL, toutes paramétrées (aucune valeur dans le texte SQL).

use super::builder::QueryBuilder;
use super::filters::MessageFilters;

/// Ajoute les conditions des filtres communs
pub(super) fn apply_filters(builder: &mut QueryBuilder, filters: &MessageFilters) {
    if filters.unread_only {
        builder.require("m.seen = 0");
    }
    if filters.flagged_only {
        builder.require("m.flagged = 1");
    }
    if filters.has_attachments {
        builder.require("m.has_attachments = 1");
    }
    if let Some(needle) = &filters.from_contains {
        apply_address_filter(builder, &["m.from_json"], needle);
    }
    if let Some(since) = filters.since {
        let bound = builder.bind(since.as_millis());
        builder.require(format!("m.sort_date >= {bound}"));
    }
    if let Some(until) = filters.until {
        let bound = builder.bind(until.as_millis());
        builder.require(format!("m.sort_date <= {bound}"));
    }
}

/// Exige qu'une adresse ou un nom d'une des colonnes JSON contienne le texte, sans tenir compte de la casse
pub(super) fn apply_address_filter(builder: &mut QueryBuilder, columns: &[&str], needle: &str) {
    let pattern = builder.bind(like_pattern(needle));
    let alternatives: Vec<String> = columns
        .iter()
        .map(|column| address_match(column, &pattern))
        .collect();
    builder.require(format!("({})", alternatives.join(" OR ")))
}

/// Condition « une entrée de la colonne JSON correspond au motif »
fn address_match(column: &str, pattern: &str) -> String {
    format!(
        "EXISTS (SELECT 1 FROM json_each({column}) j WHERE json_extract(j.value, '$.address') LIKE {pattern} ESCAPE '\\' OR json_extract(j.value, '$.name') LIKE {pattern} ESCAPE '\\')"
    )
}

/// Motif `LIKE` qui contient le texte, ses caractères spéciaux étant neutralisés
fn like_pattern(needle: &str) -> String {
    let escaped = needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}
