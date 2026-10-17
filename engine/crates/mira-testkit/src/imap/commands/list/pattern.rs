//! Motifs de LIST : `*` correspond à tout, `%` à tout sauf le séparateur.

/// Indique si le nom de boîte correspond au motif
pub(crate) fn glob_match(pattern: &str, name: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let name: Vec<char> = name.chars().collect();
    glob(&pattern, &name)
}

/// Correspondance récursive entre motif et nom
fn glob(pattern: &[char], name: &[char]) -> bool {
    match pattern.split_first() {
        None => name.is_empty(),
        Some(('*', rest)) => (0..=name.len()).any(|cut| glob(rest, &name[cut..])),
        Some(('%', rest)) => (0..=name.len())
            .any(|cut| name[..cut].iter().all(|c| *c != '/') && glob(rest, &name[cut..])),
        Some((wanted, rest)) => name.first() == Some(wanted) && glob(rest, &name[1..]),
    }
}
