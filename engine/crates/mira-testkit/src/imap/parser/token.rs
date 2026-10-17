//! Jetons d'une commande IMAP.

/// Élément lexical d'une commande : atome, chaîne, littéral ou liste.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Atom(String),
    Quoted(String),
    Literal(Vec<u8>),
    List(Vec<Token>),
}

impl Token {
    /// Texte d'un atome, d'une chaîne entre guillemets ou d'un littéral
    pub(crate) fn text(&self) -> Option<String> {
        match self {
            Self::Atom(text) | Self::Quoted(text) => Some(text.clone()),
            Self::Literal(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
            Self::List(_) => None,
        }
    }

    /// Contenu d'une liste parenthésée
    pub(crate) fn list(&self) -> Option<&[Token]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }

    /// Texte en majuscules, utile pour comparer des mots-clés
    pub(crate) fn keyword(&self) -> Option<String> {
        self.text().map(|text| text.to_ascii_uppercase())
    }
}
