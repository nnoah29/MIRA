//! Envois de test : brouillon prêt, planification, `Message-ID` mémorisé, corps reçu par le faux SMTP.

use mira_core::actions::load_server_targets;
use mira_core::drafts::schedule_send;
use mira_vocabulary::{ActionId, Draft, IdempotencyKey};

use super::fixture::Fixture;
use crate::support_actions::drafts::ready_draft;

/// Brouillon d'acc-1 avec un destinataire, planifié sans date (délai d'annulation de 10 s)
pub fn schedule_ready_draft(fx: &Fixture, key: &str) -> (Draft, ActionId) {
    let draft = ready_draft(&fx.world);
    let id = schedule_send(
        &fx.world.ctx(),
        &draft.id,
        None,
        IdempotencyKey(key.to_string()),
    )
    .unwrap();
    (draft, id)
}

/// `Message-ID` mémorisé dans l'action d'envoi, avec ses chevrons
pub fn stored_message_id(fx: &Fixture, action: &ActionId) -> Option<String> {
    let targets = fx
        .world
        .db
        .read(|tx| load_server_targets(tx, action))
        .unwrap();
    targets.sent_message_id
}

/// Vérifie qu'un seul message est parti avec le `Message-ID` mémorisé et que le brouillon a disparu
pub fn assert_sent_once_and_draft_gone(fx: &Fixture, action: &ActionId, draft: &Draft) {
    let sent = received_texts(fx);
    assert_eq!(sent.len(), 1);
    let id = stored_message_id(fx, action).unwrap();
    assert!(sent[0].contains(&format!("Message-ID: {id}")));
    assert!(
        fx.world
            .db
            .read(|tx| tx.get_draft(&draft.id))
            .unwrap()
            .is_none()
    );
}

/// Messages reçus par le faux SMTP, en texte
pub fn received_texts(fx: &Fixture) -> Vec<String> {
    fx.smtp
        .received()
        .iter()
        .map(|mail| String::from_utf8_lossy(&mail.data).into_owned())
        .collect()
}
