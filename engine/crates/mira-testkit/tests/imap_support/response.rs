//! Réponse d'une commande : lignes non étiquetées et ligne de statut.

/// Réponse complète à une commande.
#[derive(Debug)]
pub struct Response {
    pub untagged: Vec<String>,
    pub status: String,
}

impl Response {
    /// Indique si le statut est OK
    pub fn is_ok(&self) -> bool {
        self.status.split_whitespace().nth(1) == Some("OK")
    }

    /// Indique si une ligne non étiquetée contient le texte
    pub fn has(&self, needle: &str) -> bool {
        self.untagged.iter().any(|line| line.contains(needle))
    }

    /// Première ligne non étiquetée qui contient le texte
    pub fn line(&self, needle: &str) -> String {
        let found = self.untagged.iter().find(|line| line.contains(needle));
        found
            .cloned()
            .unwrap_or_else(|| panic!("no line with {needle}: {self:?}"))
    }

    /// Nombre de lignes non étiquetées qui contiennent le texte
    pub fn count(&self, needle: &str) -> usize {
        self.untagged
            .iter()
            .filter(|line| line.contains(needle))
            .count()
    }
}
