//! Requête reçue par le faux serveur d'IA, telle qu'enregistrée pour les assertions.

use std::fmt;

/// Requête reçue ; `path` est la cible complète (requête incluse), noms d'en-têtes en minuscules.
#[derive(Clone, PartialEq, Eq)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl RecordedRequest {
    /// Valeur du premier en-tête portant ce nom, sans tenir compte de la casse
    pub fn header(&self, name: &str) -> Option<&str> {
        let found = self
            .headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name));
        found.map(|(_, value)| value.as_str())
    }
}

impl fmt::Debug for RecordedRequest {
    /// Montre méthode, chemin, noms d'en-têtes et taille du corps, jamais leurs valeurs
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.headers.iter().map(|(name, _)| name.as_str()).collect();
        formatter
            .debug_struct("RecordedRequest")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("header_names", &names)
            .field("body_len", &self.body.len())
            .finish()
    }
}
