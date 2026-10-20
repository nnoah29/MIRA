//! Réponse SEARCH : UID trouvés.

use super::token::Token;
use super::tokenizer::tokenize;

/// Rassemble les UID de toutes les lignes SEARCH d'une réponse, triés et sans doublon.
pub(crate) fn collect_uids(untagged: &[Vec<u8>]) -> Vec<u32> {
    let mut found: Vec<u32> = untagged
        .iter()
        .filter_map(|line| parse_search(line))
        .flatten()
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

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
