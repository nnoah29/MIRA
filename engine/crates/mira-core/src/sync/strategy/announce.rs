//! Annonce des messages intégrés : événements « nouveau message » et « message modifié ».

use mira_vocabulary::{AccountId, EventKind, Mailbox, MessageId};

use crate::rules::apply::rules_apply_in;
use crate::sync::context::SyncContext;
use crate::sync::identity::Ingested;
use crate::sync::rule_hook::silenced_by_rules;
use crate::sync::settings_hook::silenced_by_settings;

use super::headers::Announce;
/// Émet les événements des messages créés ou modifiés, sauf en mode silencieux.
pub(super) fn announce_batch(
    ctx: &SyncContext,
    mailbox: &Mailbox,
    outcomes: &[Ingested],
    mode: Announce,
) {
    if mode == Announce::Silent {
        return;
    }
    for outcome in outcomes {
        match outcome {
            Ingested::Created(message) => {
                let by_rules = rules_apply_in(mailbox.role) && silenced_by_rules(ctx, message);
                let silenced =
                    by_rules || silenced_by_labels(ctx, message) || silenced_by_settings(ctx);
                ctx.emit(EventKind::NewMessageReceived {
                    account: ctx.account.id.clone(),
                    message: message.clone(),
                    mailbox: mailbox.id.clone(),
                    silenced,
                });
            }
            Ingested::Updated(message) => ctx.emit(updated(&ctx.account.id, message)),
            Ingested::Unchanged(_) => {}
        }
    }
}

/// Événement « message modifié ».
pub fn updated(account: &AccountId, message: &MessageId) -> EventKind {
    EventKind::MessageUpdated {
        account: account.clone(),
        message: message.clone(),
    }
}

/// Vrai si un libellé du message le masque de la Réception ou coupe ses notifications
fn silenced_by_labels(ctx: &SyncContext, message: &MessageId) -> bool {
    ctx.db
        .read(|tx| tx.label_silences(message))
        .unwrap_or(false)
}
