#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règle « répondre avec un modèle » : un brouillon est préparé à l'arrivée d'un message, jamais envoyé.

mod support_sync;

use mira_core::actions::QueueContext;
use mira_core::rules::apply::apply_rules_with;
use mira_core::sync::{SyncSession, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{DraftState, MessageId};
use support_sync::mirror::ids_by_subject;
use support_sync::provider::SyncTestProvider;
use support_sync::reply_rules::{
    deliver_and_sync, drafts_of_account, mail_from, store_identity, store_reply_rule,
};
use support_sync::within;
use support_sync::world::{SyncWorld, deliver_many, open_session, start_world};

/// Monde synchronisé une fois (un message en réception), avec identité, modèle et règle de réponse
async fn world_with_reply_rule(subject: Option<&str>) -> (SyncWorld, SyncSession) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    for name in ["Sent", "Trash", "Junk", "Drafts"] {
        world.store.create_mailbox("alice", name, None);
    }
    store_identity(&world);
    store_reply_rule(&world, subject);
    deliver_many(&world, "INBOX", "Initial", 1);
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    (world, session)
}

/// Un message qui arrive donne un brouillon de réponse rendu pour son expéditeur ; le téléchargement initial, aucun
#[tokio::test]
async fn live_arrival_prepares_a_rendered_reply_draft() {
    within(async {
        let (world, mut session) = world_with_reply_rule(None).await;
        assert!(drafts_of_account(&world).is_empty());
        deliver_and_sync(&world, &mut session, "Bob Durand <bob@example.org>", "").await;
        let drafts = drafts_of_account(&world);
        assert_eq!(drafts.len(), 1);
        let draft = &drafts[0];
        assert_eq!(draft.state, DraftState::Editing);
        assert_eq!(draft.to[0].address, "bob@example.org");
        assert_eq!(draft.subject, "Re: Demande");
        assert!(
            draft
                .body_text
                .starts_with("Bonjour Bob, je suis absent le ")
        );
        assert!(draft.body_text.ends_with("({{inconnue}})."));
        assert!(draft.in_reply_to.is_some());
    })
    .await;
}

/// L'objet du modèle, s'il en a un, remplace l'objet « Re: » de la réponse
#[tokio::test]
async fn template_subject_replaces_the_reply_subject() {
    within(async {
        let (world, mut session) = world_with_reply_rule(Some("Absent : {{objet}}")).await;
        deliver_and_sync(&world, &mut session, "Bob <bob@example.org>", "").await;
        assert_eq!(drafts_of_account(&world)[0].subject, "Absent : Demande");
    })
    .await;
}

/// Aucune réponse n'est préparée à une infolettre, à un expéditeur automatique ni à un expéditeur absent
#[tokio::test]
async fn no_draft_for_newsletters_and_automated_senders() {
    within(async {
        let (world, mut session) = world_with_reply_rule(None).await;
        let unsubscribe = "List-Unsubscribe: <mailto:stop@example.org>\r\n";
        deliver_and_sync(&world, &mut session, "Bob <bob@example.org>", unsubscribe).await;
        assert!(drafts_of_account(&world).is_empty());
        let raw = mail_from("NoReply <No-Reply@example.org>", "", "Auto", "a1@x");
        world.store.deliver("alice", "INBOX", &raw, &[]);
        sync_account(&mut session).await.unwrap();
        assert!(drafts_of_account(&world).is_empty());
    })
    .await;
}

/// Rejouer l'évaluation d'un message (reprise après arrêt brutal) ne crée pas un second brouillon
#[tokio::test]
async fn replaying_the_message_does_not_duplicate_the_draft() {
    within(async {
        let (world, mut session) = world_with_reply_rule(None).await;
        deliver_and_sync(&world, &mut session, "Bob <bob@example.org>", "").await;
        let message: MessageId = ids_by_subject(&world.db)["Demande"].clone();
        let queue = QueueContext {
            db: &world.ctx.db,
            clock: world.ctx.clock.as_ref(),
            ids: world.ctx.ids.as_ref(),
            events: world.ctx.events.as_ref(),
        };
        let quirks = world.ctx.provider.descriptor().quirks.clone();
        apply_rules_with(&queue, &quirks, &message).unwrap();
        assert_eq!(drafts_of_account(&world).len(), 1);
    })
    .await;
}

/// Un modèle supprimé rend la règle invalide : plus aucun brouillon
#[tokio::test]
async fn deleted_template_stops_the_replies() {
    within(async {
        let (world, mut session) = world_with_reply_rule(None).await;
        let id = mira_vocabulary::TemplateId::from_raw("tpl-1");
        world.db.write(|tx| tx.delete_template(&id)).unwrap();
        deliver_and_sync(&world, &mut session, "Bob <bob@example.org>", "").await;
        assert!(drafts_of_account(&world).is_empty());
    })
    .await;
}
