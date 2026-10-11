//! Jeton d'une réponse IMAP : atome, chaîne (entre guillemets ou littéral) ou liste.

/// Élément d'une réponse découpée ; les chaînes gardent leurs octets bruts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Atom(String),
    Text(Vec<u8>),
    List(Vec<Token>),
}

impl Token {
    /// Texte d'un atome, tel quel.
    pub(crate) fn atom(&self) -> Option<&str> {
        match self {
            Self::Atom(text) => Some(text),
            _ => None,
        }
    }

    /// Atome en majuscules, pour comparer les mots-clés.
    pub(crate) fn keyword(&self) -> Option<String> {
        self.atom().map(str::to_ascii_uppercase)
    }

    /// Contenu d'un atome ou d'une chaîne, converti en texte ; `NIL` donne `None`.
    pub(crate) fn text(&self) -> Option<String> {
        match self {
            Self::Atom(text) if text.eq_ignore_ascii_case("NIL") => None,
            Self::Atom(text) => Some(text.clone()),
            Self::Text(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
            Self::List(_) => None,
        }
    }

    /// Éléments d'une liste.
    pub(crate) fn list(&self) -> Option<&[Token]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }

    /// Entier non signé écrit dans un atome.
    pub(crate) fn number(&self) -> Option<u64> {
        self.atom()?.parse().ok()
    }
}
