//! Erreur de syntaxe d'une commande.

/// Erreur d'analyse syntaxique renvoyée au client sous forme de `BAD`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParseError {
    pub message: String,
}

impl ParseError {
    /// Construit une erreur à partir de son message
    pub(crate) fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
