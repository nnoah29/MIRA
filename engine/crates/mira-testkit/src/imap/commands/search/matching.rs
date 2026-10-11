//! Évaluation d'un critère de recherche sur un message.

use super::matching_text::matches_text;
use crate::imap::parser::search::Criterion;
use crate::imap::parser::uidset::resolve_set;
use crate::imap::store::data::Entry;

/// Contexte d'évaluation : UID de la boîte et position du message.
#[derive(Debug)]
pub(crate) struct MatchCtx<'a> {
    pub uids: &'a [u32],
}

/// Indique si le message possède le drapeau
fn has_flag(entry: &Entry, flag: &str) -> bool {
    entry
        .flags
        .iter()
        .any(|known| known.eq_ignore_ascii_case(flag))
}

/// Indique si l'UID du message appartient à l'ensemble (UID ou numéros de séquence)
fn in_set(set: &str, context: &MatchCtx, by_uid: bool, uid: u32) -> bool {
    resolve_set(set, context.uids, by_uid).is_ok_and(|chosen| chosen.contains(&uid))
}

/// Évalue un critère sur un message
pub(crate) fn matches(criterion: &Criterion, entry: &Entry, context: &MatchCtx) -> bool {
    let day = entry.internal_date.div_euclid(86_400);
    let size = entry.message.raw.len() as u64;
    match criterion {
        Criterion::All => true,
        Criterion::Since(limit) => day >= *limit,
        Criterion::Before(limit) => day < *limit,
        Criterion::On(wanted) => day == *wanted,
        Criterion::Uid(set) => in_set(set, context, true, entry.uid),
        Criterion::Seqset(set) => in_set(set, context, false, entry.uid),
        Criterion::ModSeq(minimum) => entry.modseq >= *minimum,
        Criterion::Flag(flag, present) => has_flag(entry, flag) == *present,
        Criterion::Larger(limit) => size > *limit,
        Criterion::Smaller(limit) => size < *limit,
        Criterion::Not(inner) => !matches(inner, entry, context),
        Criterion::Or(left, right) => {
            matches(left, entry, context) || matches(right, entry, context)
        }
        Criterion::And(list) => list.iter().all(|inner| matches(inner, entry, context)),
        other => matches_text(other, entry),
    }
}
