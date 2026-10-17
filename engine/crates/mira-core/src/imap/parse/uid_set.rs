//! Lecture des ensembles d'UID (`1:3,7`) renvoyés par le serveur.

const MAX_EXPANDED: usize = 1_000_000;

/// UID développés d'un ensemble ; `incomplete` dit que l'ensemble était trop grand ou contenait `*`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct UidSet {
    pub uids: Vec<u32>,
    pub incomplete: bool,
}

/// Étend un intervalle `a:b` ou un UID seul ; une borne `*` ou un dépassement de la borne globale marque l'ensemble incomplet.
fn expand_part(part: &str, set: &mut UidSet) {
    let (first, last) = part.split_once(':').unwrap_or((part, part));
    let (Ok(first), Ok(last)) = (first.parse::<u32>(), last.parse::<u32>()) else {
        set.incomplete = true;
        set.uids.extend(first.parse::<u32>().ok());
        return;
    };
    let (low, high) = (first.min(last), first.max(last));
    for uid in low..=high {
        if set.uids.len() >= MAX_EXPANDED {
            set.incomplete = true;
            return;
        }
        set.uids.push(uid);
    }
}

/// Étend un ensemble d'UID en liste triée sans doublon, en signalant ce qui n'a pas pu l'être.
pub(crate) fn parse_uid_set(text: &str) -> UidSet {
    let mut set = UidSet::default();
    for part in text
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
    {
        expand_part(part, &mut set);
    }
    set.uids.sort_unstable();
    set.uids.dedup();
    set
}
