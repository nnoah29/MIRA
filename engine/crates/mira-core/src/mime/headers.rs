//! Analyse des en-têtes seuls (étape 2 de la descente).

use mail_parser::MessageParser;
use mira_vocabulary::Unsubscribe;

use super::model::HeaderSummary;
use super::summary::summarize;
use super::unsubscribe::unsubscribe_from;

/// Résume des en-têtes bruts ; un résumé vide est rendu si rien n'est lisible.
pub fn parse_headers(header_bytes: &[u8]) -> HeaderSummary {
    MessageParser::default()
        .parse_headers(header_bytes)
        .map(|message| summarize(&message))
        .unwrap_or_default()
}

/// Lit le désabonnement annoncé dans des en-têtes bruts.
pub fn parse_unsubscribe_headers(header_bytes: &[u8]) -> Option<Unsubscribe> {
    let message = MessageParser::default().parse_headers(header_bytes)?;
    unsubscribe_from(&message)
}
