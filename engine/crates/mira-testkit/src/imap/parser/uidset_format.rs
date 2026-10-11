//! Écriture compacte d'une liste d'UID triés (`1:3,5`).

/// Écrit un intervalle ou un UID isolé
fn format_range(first: u32, last: u32) -> String {
    if first == last {
        first.to_string()
    } else {
        format!("{first}:{last}")
    }
}

/// Écrit une liste d'UID triés en ensemble compact
pub(crate) fn format_uid_set(uids: &[u32]) -> String {
    let mut iter = uids.iter().copied();
    let Some(mut first) = iter.next() else {
        return String::new();
    };
    let (mut last, mut parts) = (first, Vec::new());
    for uid in iter {
        if uid == last + 1 {
            last = uid;
        } else {
            parts.push(format_range(first, last));
            (first, last) = (uid, uid);
        }
    }
    parts.push(format_range(first, last));
    parts.join(",")
}
