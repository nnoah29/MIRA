//! Réponses d'ouverture d'une boîte (SELECT) : marqueurs, présence, changements.

use super::fetch::parse_fetch;
use super::fetched::fetched_flags;
use super::status::{ResponseCode, untagged_status};
use super::token::Token;
use super::tokenizer::tokenize;
use super::vanished::parse_vanished;
use crate::imap::types::SelectedMailbox;

/// Applique un code de réponse (`UIDVALIDITY`, `UIDNEXT`, `HIGHESTMODSEQ`) à la boîte.
fn apply_code(selected: &mut SelectedMailbox, code: &ResponseCode) {
    let value = code.args.split_whitespace().next();
    match (code.name.as_str(), value) {
        ("UIDVALIDITY", Some(text)) => selected.uid_validity = text.parse().unwrap_or(0),
        ("UIDNEXT", Some(text)) => selected.uid_next = text.parse().unwrap_or(0),
        ("HIGHESTMODSEQ", Some(text)) => selected.highest_modseq = text.parse().ok(),
        _ => {}
    }
}

/// Applique les lignes `n EXISTS` et `FLAGS (...)`.
fn apply_counter(selected: &mut SelectedMailbox, line: &[u8]) {
    match tokenize(line).as_slice() {
        [number, Token::Atom(word)] if word.eq_ignore_ascii_case("EXISTS") => {
            selected.exists = number
                .number()
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(0);
        }
        [Token::Atom(word), Token::List(flags)] if word.eq_ignore_ascii_case("FLAGS") => {
            selected.flags = flags.iter().filter_map(Token::text).collect();
        }
        _ => {}
    }
}

/// Applique une ligne non étiquetée de la réponse à la boîte en construction.
fn apply_line(selected: &mut SelectedMailbox, line: &[u8]) {
    if let Some(data) = parse_fetch(line) {
        selected.changed.extend(fetched_flags(&data));
    } else if let Some(gone) = parse_vanished(line) {
        selected.vanished.extend(gone);
    } else if let Some((_, code, _)) = untagged_status(line) {
        if let Some(code) = code {
            apply_code(selected, &code);
        }
    } else {
        apply_counter(selected, line);
    }
}

/// Construit la description de la boîte à partir des lignes non étiquetées d'un SELECT.
pub(crate) fn parse_select(lines: &[Vec<u8>]) -> SelectedMailbox {
    let mut selected = SelectedMailbox::default();
    for line in lines {
        apply_line(&mut selected, line);
    }
    selected.vanished.sort_unstable();
    selected.vanished.dedup();
    selected
}
