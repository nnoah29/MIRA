//! Réponses que le faux serveur d'IA peut jouer, mises en file par les tests.

/// Comportement joué pour une requête ; le dernier de la file se répète.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Behavior {
    /// Flux `text/event-stream` : un morceau HTTP par élément, avec un délai entre deux morceaux.
    Stream {
        chunks: Vec<Vec<u8>>,
        chunk_delay_ms: u64,
    },
    /// Réponse complète avec ce code, ces en-têtes et ce corps (JSON par défaut).
    Status {
        code: u16,
        headers: Vec<(String, String)>,
        body: String,
    },
    /// Requête lue puis silence total jusqu'à l'arrêt du serveur.
    Hang,
    /// Début de flux puis fermeture sans morceau final.
    DropAfter { chunks: Vec<Vec<u8>> },
    /// Redirection 302 vers cette adresse.
    Redirect { to: String },
    /// Liste de modèles : réponse 200 `application/json`.
    Models { json: String },
    /// Flux de commentaires SSE d'exactement cette taille, sans `Content-Length`.
    Oversize { bytes: usize },
}
