//! Composition des notifications d'un compte : une par message jusqu'à trois, un résumé au-delà.

use mira_assembly::mira_vocabulary::NotificationSettings;

use super::{Arrival, Notification, Texts};

const MAX_INDIVIDUAL: usize = 3;

/// Identifiant stable d'une notification (FNV-1a), pour qu'une même notification se remplace elle-même.
fn stable_id(text: &str) -> u32 {
    let hash = text.bytes().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    });
    hash & 0x7fff_ffff
}

/// Notification d'un seul message : expéditeur et objet avec aperçu, texte générique sans.
fn single(arrival: &Arrival, settings: &NotificationSettings, texts: &Texts) -> Notification {
    let (title, body) = if settings.preview {
        (arrival.sender.clone(), arrival.subject.clone())
    } else {
        (texts.new_message.clone(), arrival.address.clone())
    };
    Notification {
        id: stable_id(&arrival.message),
        title,
        body,
        sound: settings.sound,
        message: Some(arrival.message.clone()),
    }
}

/// Compose les notifications des arrivées d'un même compte ; au-delà de trois, un résumé qui ouvre le premier message.
pub fn compose(
    arrivals: &[Arrival],
    settings: &NotificationSettings,
    texts: &Texts,
) -> Vec<Notification> {
    let Some(first) = arrivals.first() else {
        return Vec::new();
    };
    if arrivals.len() <= MAX_INDIVIDUAL {
        return arrivals
            .iter()
            .map(|arrival| single(arrival, settings, texts))
            .collect();
    }
    vec![Notification {
        id: stable_id(&first.account),
        title: texts.many(arrivals.len()),
        body: first.address.clone(),
        sound: settings.sound,
        message: Some(first.message.clone()),
    }]
}
