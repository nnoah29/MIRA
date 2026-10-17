//! Liste blanche des ressources chargées automatiquement : seules les images incorporées passent.

const AUTO_LOADING_ATTRIBUTES: &[(&str, &str)] = &[
    ("img", "src"),
    ("img", "srcset"),
    ("image", "href"),
    ("input", "src"),
    ("video", "poster"),
    ("audio", "src"),
    ("video", "src"),
    ("source", "src"),
    ("source", "srcset"),
    ("track", "src"),
    ("table", "background"),
    ("td", "background"),
    ("th", "background"),
    ("tr", "background"),
    ("body", "background"),
];

/// Indique si cet attribut charge automatiquement une ressource
fn is_auto_loading(element: &str, attribute: &str) -> bool {
    AUTO_LOADING_ATTRIBUTES
        .iter()
        .any(|(tag, name)| *tag == element && *name == attribute)
}

/// Met une adresse sous la forme que lit un navigateur (sans tabulation ni saut de ligne, `\` devient `/`)
fn normalize_url(value: &str) -> String {
    let kept: String = value
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .map(|c| if c == '\\' { '/' } else { c })
        .collect();
    kept.trim_matches(|c: char| c <= ' ').to_ascii_lowercase()
}

/// Indique si l'adresse est une ressource incorporée au message (`cid:` ou image `data:` hors SVG)
fn is_embedded_resource(value: &str) -> bool {
    let normalized = normalize_url(value);
    if normalized.starts_with("cid:") {
        return true;
    }
    normalized
        .strip_prefix("data:image/")
        .is_some_and(|rest| !rest.starts_with("svg"))
}

/// Indique si cette valeur doit être retirée tant que le contenu distant n'est pas autorisé
pub(crate) fn is_blocked_until_allowed(element: &str, attribute: &str, value: &str) -> bool {
    if !is_auto_loading(element, attribute) {
        return false;
    }
    attribute == "srcset" || !is_embedded_resource(value)
}

/// Indique si un lien porterait un contenu `data:` que le lecteur afficherait
pub(crate) fn is_data_link(attribute: &str, value: &str) -> bool {
    attribute == "href" && normalize_url(value).starts_with("data:")
}
