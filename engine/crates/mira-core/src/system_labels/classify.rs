//! Verdict pur sur le dernier message d'un fil : « en attente » ou « mention », et quand le réévaluer.

use super::me::Me;
use super::mention::mentions_me;

/// Délai sans réponse avant qu'un message envoyé passe « en attente » : 2 jours.
pub const WAITING_AFTER_MS: i64 = 2 * 24 * 3600 * 1000;
/// Au-delà de 30 jours, un fil n'est plus signalé.
pub const LABEL_HORIZON_MS: i64 = 30 * 24 * 3600 * 1000;

/// Ce qui compte dans le dernier message vivant d'un fil.
#[derive(Debug)]
pub struct LastMessage {
    pub from_own: bool,
    pub sender_automated: bool,
    pub awaits_human: bool,
    pub date: i64,
    pub text: String,
}

/// Verdict : les deux marqueurs détectés et l'instant où il peut changer sans nouveau message.
#[derive(Debug, PartialEq, Eq)]
pub struct Verdict {
    pub waiting: bool,
    pub mention: bool,
    pub recheck_at: Option<i64>,
}

/// Verdict d'un fil : sans message vivant ou au-delà de 30 jours, aucun marqueur
pub fn classify(last: Option<&LastMessage>, me: &Me, now: i64) -> Verdict {
    let none = Verdict {
        waiting: false,
        mention: false,
        recheck_at: None,
    };
    let Some(last) = last.filter(|l| now - l.date <= LABEL_HORIZON_MS) else {
        return none;
    };
    if last.from_own {
        return own_message_verdict(last, now);
    }
    let mention = !last.sender_automated && mentions_me(&last.text, me);
    Verdict {
        mention,
        recheck_at: mention.then_some(last.date + LABEL_HORIZON_MS),
        ..none
    }
}

/// Verdict quand le dernier message est de moi : en attente après le délai, s'il vise quelqu'un de réel
fn own_message_verdict(last: &LastMessage, now: i64) -> Verdict {
    let due = last.date + WAITING_AFTER_MS;
    let waiting = last.awaits_human && now >= due;
    let recheck_at = match (last.awaits_human, waiting) {
        (false, _) => None,
        (true, true) => Some(last.date + LABEL_HORIZON_MS),
        (true, false) => Some(due),
    };
    Verdict {
        waiting,
        mention: false,
        recheck_at,
    }
}
