//! Textes fixes des notifications : français si le système l'est, anglais sinon.

/// Textes que le moteur écrit lui-même ; `{n}` est remplacé par le nombre de messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texts {
    pub new_message: String,
    pub many_new_messages: String,
    pub keyring_locked_title: String,
    pub keyring_locked_body: String,
}

impl Texts {
    /// Textes de la langue du système (`fr…` donne le français, tout le reste l'anglais).
    pub fn for_locale(locale: &str) -> Self {
        let fields = if locale.starts_with("fr") {
            [
                "Nouveau message",
                "{n} nouveaux messages",
                "MIRA attend le déverrouillage du trousseau",
                "Déverrouillez votre trousseau de clés pour reprendre la synchronisation.",
            ]
        } else {
            [
                "New message",
                "{n} new messages",
                "MIRA is waiting for your keyring",
                "Unlock your keyring to resume syncing.",
            ]
        };
        let [single, many, locked_title, locked_body] = fields.map(str::to_string);
        Self {
            new_message: single,
            many_new_messages: many,
            keyring_locked_title: locked_title,
            keyring_locked_body: locked_body,
        }
    }

    /// Titre du résumé de `count` messages.
    pub fn many(&self, count: usize) -> String {
        self.many_new_messages.replace("{n}", &count.to_string())
    }
}
