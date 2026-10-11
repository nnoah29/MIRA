//! Lecture des ensembles d'UID (`1:3,7`) renvoyés par le serveur.

const MAX_EXPANDED: usize = 1_000_000;

/// Étend un intervalle `a:b` ou un UID seul, sans dépasser la borne globale.
fn expand_part(part: &str, out: &mut Vec<u32>) {
    let (first, last) = part.split_once(':').unwrap_or((part, part));
    let (Ok(first), Ok(last)) = (first.parse::<u32>(), last.parse::<u32>()) else {
        return;
    };
    let (low, high) = (first.min(last), first.max(last));
    for uid in low..=high {
        if out.len() >= MAX_EXPANDED {
            return;
        }
        out.push(uid);
    }
}

/// Étend un ensemble d'UID en liste triée sans doublon.
pub(crate) fn parse_uid_set(text: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for part in text.split(',') {
        expand_part(part.trim(), &mut out);
    }
    out.sort_unstable();
    out.dedup();
    out
}
