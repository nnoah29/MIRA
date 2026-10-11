//! Repérage des ressources distantes (pixels espions, images hébergées) dans les attributs.

const REMOTE_ATTRIBUTES: &[(&str, &str)] = &[
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
pub(crate) fn is_auto_loading(element: &str, attribute: &str) -> bool {
    REMOTE_ATTRIBUTES
        .iter()
        .any(|(tag, name)| *tag == element && *name == attribute)
}

/// Indique si une adresse désigne une ressource réseau (et non `cid:` ou `data:`)
pub(crate) fn is_remote_url(value: &str) -> bool {
    let lowered = value.trim().to_ascii_lowercase();
    lowered.starts_with("http:")
        || lowered.starts_with("https:")
        || lowered.starts_with("//")
        || lowered.contains(" http")
}
