//! Messages auxquels une règle ne prépare jamais de brouillon de réponse : envois automatiques et listes.

use mira_vocabulary::MessageSummary;

const AUTOMATED_SENDERS: [&str; 5] = [
    "noreply",
    "no-reply",
    "donotreply",
    "do-not-reply",
    "mailer-daemon",
];

/// Vrai si l'adresse est celle d'un expéditeur automatique (partie locale `noreply`, `no-reply`…)
fn is_automated_address(address: &str) -> bool {
    let local = address.split('@').next().unwrap_or_default().to_lowercase();
    AUTOMATED_SENDERS.contains(&local.as_str())
}

/// Vrai si une réponse a un sens : ni infolettre, ni expéditeur automatique, ni expéditeur inconnu
pub(super) fn deserves_reply(message: &MessageSummary) -> bool {
    message.unsubscribe.is_none()
        && !message.from.is_empty()
        && !message
            .from
            .iter()
            .any(|sender| is_automated_address(&sender.address))
}
