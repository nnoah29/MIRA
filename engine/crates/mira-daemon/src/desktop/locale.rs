//! Langue du système, lue dans l'environnement.

/// Première valeur non vide parmi `LC_ALL`, `LC_MESSAGES` et `LANG` ; vide si aucune.
pub fn pick_locale(all: Option<&str>, messages: Option<&str>, lang: Option<&str>) -> String {
    [all, messages, lang]
        .into_iter()
        .flatten()
        .find(|value| !value.is_empty())
        .unwrap_or_default()
        .to_string()
}

/// Langue du système d'après l'environnement du processus.
pub fn system_locale() -> String {
    let read = |name: &str| std::env::var(name).ok();
    pick_locale(
        read("LC_ALL").as_deref(),
        read("LC_MESSAGES").as_deref(),
        read("LANG").as_deref(),
    )
}
