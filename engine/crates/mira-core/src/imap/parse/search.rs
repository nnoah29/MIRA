//! Réponse SEARCH : UID trouvés.

use super::token::Token;
use super::tokenizer::tokenize;

/// Lit une ligne `SEARCH n n n` et rend les identifiants triés ; `None` si ce n'est pas un SEARCH.
pub(crate) fn parse_search(line: &[u8]) -> Option<Vec<u32>> {
    let tokens = tokenize(line);
    let (first, rest) = tokens.split_first()?;
    (first.keyword()? == "SEARCH").then_some(())?;
    let mut found: Vec<u32> = rest
        .iter()
        .filter_map(Token::number)
        .filter_map(|number| u32::try_from(number).ok())
        .collect();
    found.sort_unstable();
    found.dedup();
    Some(found)
}
