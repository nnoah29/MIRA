//! Réponse LIST : boîtes du compte avec leurs attributs.

use mira_vocabulary::RawMailbox;

use super::token::Token;
use super::tokenizer::tokenize;

/// Lit une ligne `LIST (attributs) "séparateur" nom` ; `None` si ce n'est pas un LIST.
pub(crate) fn parse_list(line: &[u8]) -> Option<RawMailbox> {
    let mut tokens = tokenize(line).into_iter();
    (tokens.next()?.keyword()? == "LIST").then_some(())?;
    let attributes = tokens
        .next()?
        .list()?
        .iter()
        .filter_map(Token::text)
        .collect();
    let delimiter = tokens.next()?.text();
    let name = tokens.next()?.text()?;
    Some(RawMailbox {
        name,
        delimiter,
        attributes,
    })
}
