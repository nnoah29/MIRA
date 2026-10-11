//! Réponse FETCH : paires (nom d'attribut, valeur) d'un message.

use super::token::Token;
use super::tokenizer::tokenize;

/// Attributs d'une réponse `n FETCH (...)`, noms en majuscules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FetchData {
    pub items: Vec<(String, Token)>,
}

/// Analyse une ligne `n FETCH (...)` ; `None` si ce n'en est pas une.
pub(crate) fn parse_fetch(line: &[u8]) -> Option<FetchData> {
    let mut tokens = tokenize(line).into_iter();
    tokens.next()?.number()?;
    (tokens.next()?.keyword()? == "FETCH").then_some(())?;
    let Token::List(list) = tokens.next()? else {
        return None;
    };
    let mut items = Vec::new();
    let mut parts = list.into_iter();
    while let Some(name) = parts.next() {
        items.push((name.keyword()?, parts.next()?));
    }
    Some(FetchData { items })
}

impl FetchData {
    /// Valeur de l'attribut de ce nom (en majuscules).
    pub(crate) fn get(&self, name: &str) -> Option<&Token> {
        self.items
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    /// Identifiant unique du message.
    pub(crate) fn uid(&self) -> Option<u32> {
        u32::try_from(self.get("UID")?.number()?).ok()
    }

    /// Drapeaux du message, tels que le serveur les écrit.
    pub(crate) fn flags(&self) -> Vec<String> {
        let list = self.get("FLAGS").and_then(Token::list).unwrap_or(&[]);
        list.iter().filter_map(Token::text).collect()
    }

    /// Numéro de modification (CONDSTORE).
    pub(crate) fn modseq(&self) -> Option<u64> {
        self.get("MODSEQ")?.list()?.first()?.number()
    }

    /// Taille du message en octets.
    pub(crate) fn size(&self) -> u64 {
        self.get("RFC822.SIZE").and_then(Token::number).unwrap_or(0)
    }
}
