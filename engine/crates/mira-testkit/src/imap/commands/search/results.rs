//! Écriture de la réponse SEARCH ou ESEARCH.

use super::run::Hit;
use crate::imap::parser::command::Command;
use crate::imap::parser::search::SearchQuery;
use crate::imap::parser::uidset_format::format_uid_set;
use crate::imap::response::quoting::quote;

/// Ligne de résultat selon la présence d'une clause RETURN
pub(crate) fn format_results(
    hits: &[Hit],
    query: &SearchQuery,
    command: &Command,
    uses_modseq: bool,
) -> String {
    match &query.returns {
        Some(options) => format_esearch(hits, options, command),
        None => format_plain(hits, uses_modseq),
    }
}

/// Réponse `SEARCH` classique, avec le plus grand MODSEQ si MODSEQ est utilisé
fn format_plain(hits: &[Hit], uses_modseq: bool) -> String {
    let mut line = "SEARCH".to_string();
    for hit in hits {
        line.push_str(&format!(" {}", hit.id));
    }
    let highest = hits.iter().map(|hit| hit.modseq).max();
    if let (true, Some(modseq)) = (uses_modseq, highest) {
        line.push_str(&format!(" (MODSEQ {modseq})"));
    }
    line
}

/// Réponse `ESEARCH` limitée aux options demandées (ALL par défaut)
fn format_esearch(hits: &[Hit], options: &[String], command: &Command) -> String {
    let mut line = format!("ESEARCH (TAG {})", quote(&command.tag));
    if command.uid {
        line.push_str(" UID");
    }
    let ids: Vec<u32> = hits.iter().map(|hit| hit.id).collect();
    let all = ["ALL".to_string()];
    for option in if options.is_empty() {
        &all[..]
    } else {
        options
    } {
        match (option.as_str(), ids.first(), ids.last()) {
            ("MIN", Some(min), _) => line.push_str(&format!(" MIN {min}")),
            ("MAX", _, Some(max)) => line.push_str(&format!(" MAX {max}")),
            ("COUNT", _, _) => line.push_str(&format!(" COUNT {}", ids.len())),
            ("ALL", Some(_), _) => line.push_str(&format!(" ALL {}", format_uid_set(&ids))),
            _ => {}
        }
    }
    line
}
