//! Analyse d'une commande complète : étiquette, nom, préfixe UID et arguments.

use std::vec::IntoIter;

use super::error::ParseError;
use super::token::Token;
use super::tokenizer::tokenize;

/// Commande client analysée.
#[derive(Debug, Clone)]
pub(crate) struct Command {
    pub tag: String,
    pub name: String,
    pub uid: bool,
    pub args: Vec<Token>,
    pub line: String,
}

/// Analyse une commande brute ; `line` est sa première ligne pour le journal
pub(crate) fn parse_command(raw: &[u8], line: &str) -> Result<Command, ParseError> {
    let mut tokens = tokenize(raw)?.into_iter();
    let tag = next_word(&mut tokens, "missing tag")?;
    let mut name = next_word(&mut tokens, "missing command")?.to_ascii_uppercase();
    let uid = name == "UID";
    if uid {
        name = next_word(&mut tokens, "missing UID command")?.to_ascii_uppercase();
    }
    Ok(Command {
        tag,
        name,
        uid,
        args: tokens.collect(),
        line: line.to_string(),
    })
}

/// Prend le jeton suivant sous forme de texte
fn next_word(tokens: &mut IntoIter<Token>, message: &str) -> Result<String, ParseError> {
    tokens
        .next()
        .and_then(|token| token.text())
        .ok_or_else(|| ParseError::new(message))
}
