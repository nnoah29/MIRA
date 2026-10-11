//! Règles de forme d'une adresse e-mail (partie locale et domaine).

/// Sépare `local@domaine` si la forme est valide : un seul `@`, deux parties acceptables.
pub fn split_address(text: &str) -> Option<(&str, &str)> {
    let (local, domain) = text.split_once('@')?;
    let local_ok = !local.is_empty() && !local.contains(char::is_whitespace);
    (local_ok && domain_is_valid(domain)).then_some((local, domain))
}

/// Vérifie qu'un domaine est non vide, pointé, sans espace ni point en bordure ni `..`.
pub fn domain_is_valid(domain: &str) -> bool {
    domain.contains('.')
        && !domain.contains('@')
        && !domain.contains(char::is_whitespace)
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}
