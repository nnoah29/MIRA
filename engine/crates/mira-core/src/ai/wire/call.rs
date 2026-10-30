//! Appel HTTP décrit par un format : chemin, en-têtes et corps, sans la clé d'authentification.

/// Appel prêt à émettre ; le chemin est relatif à l'adresse de base de la connexion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpCall {
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}
