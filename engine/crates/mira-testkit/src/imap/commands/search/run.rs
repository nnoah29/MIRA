//! Exécution d'une recherche sur une boîte.

use super::matching::{MatchCtx, matches};
use crate::imap::parser::search::Criterion;
use crate::imap::store::data::StoreData;

/// Message trouvé : identifiant à renvoyer (UID ou numéro) et MODSEQ.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Hit {
    pub id: u32,
    pub modseq: u64,
}

/// Indique si l'un des critères est un critère MODSEQ
pub(crate) fn contains_modseq(criteria: &[Criterion]) -> bool {
    criteria.iter().any(|criterion| match criterion {
        Criterion::ModSeq(_) => true,
        Criterion::Not(inner) => contains_modseq(std::slice::from_ref(inner)),
        Criterion::Or(left, right) => {
            contains_modseq(std::slice::from_ref(left))
                || contains_modseq(std::slice::from_ref(right))
        }
        Criterion::And(list) => contains_modseq(list),
        _ => false,
    })
}

/// Messages de la boîte qui satisfont tous les critères
pub(crate) fn run_search(
    data: &StoreData,
    user: &str,
    mailbox: &str,
    criteria: &[Criterion],
    by_uid: bool,
) -> Vec<Hit> {
    let Some(found) = data.mailbox(user, mailbox) else {
        return Vec::new();
    };
    let uids = found.uids();
    let mut hits = Vec::new();
    for (index, entry) in found.entries.iter().enumerate() {
        let context = MatchCtx { uids: &uids };
        if criteria
            .iter()
            .all(|criterion| matches(criterion, entry, &context))
        {
            let id = if by_uid { entry.uid } else { index as u32 + 1 };
            hits.push(Hit {
                id,
                modseq: entry.modseq,
            });
        }
    }
    hits
}
