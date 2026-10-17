//! Journalisation des commandes sans secret et clés de pannes injectables.

use crate::imap::parser::command::Command;
use crate::imap::parser::token::Token;

/// Ligne de journal d'une commande ; les mots de passe sont masqués
pub(crate) fn log_line(command: &Command) -> String {
    let first = command
        .args
        .first()
        .and_then(Token::text)
        .unwrap_or_default();
    match command.name.as_str() {
        "LOGIN" => format!("LOGIN {first} ***"),
        "AUTHENTICATE" => format!("AUTHENTICATE {} ***", first.to_ascii_uppercase()),
        _ => command
            .line
            .split_once(' ')
            .map_or("", |(_, rest)| rest)
            .trim()
            .to_string(),
    }
}

/// Clés sous lesquelles une panne peut être programmée pour cette commande
pub(crate) fn failure_keys(command: &Command) -> Vec<String> {
    let mut keys = vec![command.name.clone()];
    if command.uid {
        keys.push(format!("UID {}", command.name));
    }
    keys
}
