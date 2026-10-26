//! Monde de test des règles qui répondent avec un modèle : identité d'envoi, modèle et courrier d'un expéditeur choisi.

use mira_vocabulary::{
    AccountId, DraftId, EmailAddress, Identity, IdentityId, MessageTemplate, RuleAction, TemplateId,
};

use mira_core::sync::{SyncSession, sync_account};

use super::mirror::account;
use super::rules::{catch_all, store_rule};
use super::world::SyncWorld;

/// Courrier envoyé par `from`, avec des en-têtes supplémentaires
pub fn mail_from(from: &str, extra: &str, subject: &str, message_id: &str) -> Vec<u8> {
    format!(
        "From: {from}\r\nTo: alice@example.org\r\nSubject: {subject}\r\nMessage-ID: <{message_id}>\r\n{extra}Date: Mon, 01 Jan 2024 10:00:00 +0000\r\n\r\nCorps\r\n"
    )
    .into_bytes()
}

/// Enregistre l'identité d'envoi du compte de test
pub fn store_identity(world: &SyncWorld) {
    let identity = Identity {
        id: IdentityId::from_raw("id-1"),
        account: account(),
        address: EmailAddress::parse("alice@example.org").unwrap(),
        display_name: "Alice Martin".to_string(),
        signature: None,
        is_default: true,
    };
    world.db.write(|tx| tx.insert_identity(&identity)).unwrap();
}

/// Enregistre un modèle d'identifiant `tpl-1` et la règle « tout message → répondre avec ce modèle »
pub fn store_reply_rule(world: &SyncWorld, subject: Option<&str>) {
    let template = MessageTemplate {
        id: TemplateId::from_raw("tpl-1"),
        name: "Absence".to_string(),
        subject: subject.map(str::to_string),
        body: "Bonjour {{contact.prénom}}, je suis absent le {{date}} ({{inconnue}}).".to_string(),
        created_at: 1,
        updated_at: 1,
    };
    world.db.write(|tx| tx.insert_template(&template)).unwrap();
    let action = RuleAction::ReplyWithTemplate {
        template: template.id,
    };
    store_rule(world, &catch_all("r-reply", vec![action]));
}

/// Brouillons du compte de test
pub fn drafts_of_account(world: &SyncWorld) -> Vec<mira_vocabulary::Draft> {
    let id: AccountId = account();
    world.db.read(|tx| tx.list_drafts(&id)).unwrap()
}

/// Identifiants des brouillons du compte de test
pub fn draft_ids(world: &SyncWorld) -> Vec<DraftId> {
    drafts_of_account(world).into_iter().map(|d| d.id).collect()
}

/// Un message livré par `from` avec ces en-têtes en plus, puis une synchronisation
pub async fn deliver_and_sync(
    world: &SyncWorld,
    session: &mut SyncSession,
    from: &str,
    extra: &str,
) {
    let raw = mail_from(from, extra, "Demande", "d1@x");
    world.store.deliver("alice", "INBOX", &raw, &[]);
    sync_account(session).await.unwrap();
}
