//! Faut-il notifier cette arrivée ? Réglages, verdict du moteur, sourdine, présence et heures calmes.

use mira_assembly::mira_vocabulary::{AccountId, EngineSettings, MailboxRole, QuietHours};

/// Verdict pour une arrivée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Show,
    Skip,
}

/// Dit si l'heure tombe dans les heures calmes ; la plage peut passer minuit, un début égal à la fin est vide.
pub fn in_quiet_hours(hours: Option<QuietHours>, hour: u8) -> bool {
    let Some(QuietHours { from, to }) = hours else {
        return false;
    };
    if from <= to {
        hour >= from && hour < to
    } else {
        hour >= from || hour < to
    }
}

/// Décide pour une arrivée ; `active` dit qu'un client regarde MIRA, `hour` est l'heure locale.
pub fn decide(
    settings: &EngineSettings,
    silenced: bool,
    account: &AccountId,
    active: bool,
    hour: u8,
) -> Decision {
    let notifications = &settings.notifications;
    let blocked = !notifications.enabled
        || silenced
        || settings.mutes(account)
        || (active && !notifications.when_focused)
        || in_quiet_hours(settings.quiet_hours, hour);
    if blocked {
        Decision::Skip
    } else {
        Decision::Show
    }
}

/// Dit si un message mérite une notification : non lu et arrivé en réception (ni Envoyés, ni Corbeille, ni Indésirables…).
pub fn worth_notifying(role: MailboxRole, seen: bool) -> bool {
    role == MailboxRole::Inbox && !seen
}
