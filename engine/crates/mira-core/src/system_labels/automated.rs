//! Expéditeurs automatiques : ils n'attendent pas de réponse et ne « mentionnent » personne.

const BASES: [&str; 8] = [
    "noreply",
    "donotreply",
    "notification",
    "notifications",
    "mailerdaemon",
    "postmaster",
    "bounce",
    "bounces",
];

/// L'adresse est-elle celle d'un envoi automatique (`noreply@`, `notifications@`, `bounce-12@`…) ?
pub fn is_automated_address(address: &str) -> bool {
    let local = address.split('@').next().unwrap_or("").to_lowercase();
    let stops = local
        .match_indices(['-', '_', '.', '+'])
        .map(|(index, _)| index)
        .chain(std::iter::once(local.len()));
    stops.into_iter().any(|end| is_base(&local[..end]))
}

/// Le début d'adresse, séparateurs retirés, est-il un nom d'envoi automatique ?
fn is_base(prefix: &str) -> bool {
    let squeezed: String = prefix
        .chars()
        .filter(|c| !matches!(c, '-' | '_' | '.' | '+'))
        .collect();
    BASES.contains(&squeezed.as_str())
}
