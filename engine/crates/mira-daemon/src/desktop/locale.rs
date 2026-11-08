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

/// Langue du système d'après l'environnement du processus, puis d'après les réglages de Windows.
pub fn system_locale() -> String {
    let read = |name: &str| std::env::var(name).ok();
    let from_env = pick_locale(
        read("LC_ALL").as_deref(),
        read("LC_MESSAGES").as_deref(),
        read("LANG").as_deref(),
    );
    if from_env.is_empty() {
        return windows_locale();
    }
    from_env
}

/// Langue de l'utilisateur dans le registre de Windows (`fr-FR`) ; vide si illisible.
#[cfg(windows)]
fn windows_locale() -> String {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Control Panel\International")
        .and_then(|key| key.get_value("LocaleName"))
        .unwrap_or_default()
}

/// Hors Windows, pas d'autre source que l'environnement.
#[cfg(not(windows))]
fn windows_locale() -> String {
    String::new()
}
