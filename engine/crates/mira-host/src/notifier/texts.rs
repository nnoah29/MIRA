//! Textes fixes des notifications : français si le système l'est, anglais sinon.

/// Textes que le moteur écrit lui-même ; `{n}` est remplacé par le nombre de messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texts {
    pub new_message: String,
    pub many_new_messages: String,
}

impl Texts {
    /// Textes de la langue du système (`fr…` donne le français, tout le reste l'anglais).
    pub fn for_locale(locale: &str) -> Self {
        let (single, many) = if locale.starts_with("fr") {
            ("Nouveau message", "{n} nouveaux messages")
        } else {
            ("New message", "{n} new messages")
        };
        Self {
            new_message: single.to_string(),
            many_new_messages: many.to_string(),
        }
    }

    /// Titre du résumé de `count` messages.
    pub fn many(&self, count: usize) -> String {
        self.many_new_messages.replace("{n}", &count.to_string())
    }
}
