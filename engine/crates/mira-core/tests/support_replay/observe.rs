//! Lectures de contrôle côté serveur et côté file pour les assertions du rejeu.

use super::fixture::Fixture;

/// Identifiants `Message-ID` (sans chevrons) des messages d'une boîte du serveur, dans l'ordre des UID
pub fn server_message_ids(fx: &Fixture, mailbox: &str) -> Vec<String> {
    fx.env
        .store
        .messages("alice", mailbox)
        .iter()
        .map(|message| header_value(&message.raw, "Message-ID"))
        .collect()
}

/// Valeur d'un en-tête d'un message brut, sans chevrons
fn header_value(raw: &[u8], name: &str) -> String {
    let text = String::from_utf8_lossy(raw).into_owned();
    let prefix = format!("{}:", name.to_lowercase());
    let line = text
        .lines()
        .find(|line| line.to_lowercase().starts_with(&prefix))
        .unwrap_or_default();
    line[prefix.len()..]
        .trim()
        .trim_matches(['<', '>'])
        .to_string()
}

/// Drapeaux (triés) du message de la boîte qui porte ce `Message-ID`
pub fn server_flags(fx: &Fixture, mailbox: &str, message_id: &str) -> Vec<String> {
    let found = fx
        .env
        .store
        .messages("alice", mailbox)
        .into_iter()
        .find(|message| header_value(&message.raw, "Message-ID") == message_id);
    let mut flags = found.map(|message| message.flags).unwrap_or_default();
    flags.sort();
    flags
}

/// Position de la première commande du serveur qui contient le texte, dans le journal
pub fn command_position(fx: &Fixture, needle: &str) -> Option<usize> {
    let log = fx.env.store.command_log();
    log.iter().position(|line| line.contains(needle))
}

/// Nombre de commandes du journal qui contiennent le texte
pub fn command_count(fx: &Fixture, needle: &str) -> usize {
    let log = fx.env.store.command_log();
    log.iter().filter(|line| line.contains(needle)).count()
}
