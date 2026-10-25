//! Brouillons de réponse préparés par les règles : le modèle est rendu pour l'expéditeur, jamais envoyé.

use mira_vocabulary::{
    ContactInfo, Identity, MessageSummary, MiraError, RenderContext, TemplateId, Timestamp,
};

use super::reply_clock::date_and_time;
use super::reply_eligibility::deserves_reply;
use crate::actions::QueueContext;
use crate::drafts::{RuleReplyText, create_rule_reply};
use crate::templates::render_template;

/// Expéditeur du message, tel que les variables `contact.*` le lisent
fn sender_contact(message: &MessageSummary) -> Option<ContactInfo> {
    let sender = message.from.first()?;
    let first_name = sender
        .name
        .as_deref()
        .and_then(|name| name.split_whitespace().next())
        .map(str::to_string);
    Some(ContactInfo {
        name: sender.name.clone(),
        first_name,
        email: Some(sender.address.clone()),
    })
}

/// Valeurs des variables pour une réponse à ce message, depuis l'identité d'envoi
fn reply_context(identity: &Identity, message: &MessageSummary, now: Timestamp) -> RenderContext {
    let (date, time) = date_and_time(now);
    RenderContext {
        contact: sender_contact(message),
        account_name: Some(identity.display_name.clone()),
        account_email: Some(identity.address.as_str().to_string()),
        subject: Some(message.subject.clone()),
        date: Some(date),
        time: Some(time),
    }
}

/// Prépare le brouillon de réponse d'un modèle ; un modèle supprimé entre-temps est ignoré
fn prepare_reply(
    queue: &QueueContext<'_>,
    message: &MessageSummary,
    rank: usize,
    template: &TemplateId,
) -> Result<(), MiraError> {
    let Some(found) = queue.db.read(|tx| tx.get_template(template))? else {
        return Ok(());
    };
    let now = queue.clock.now();
    create_rule_reply(queue, &message.id, rank, |identity, summary| {
        let rendered = render_template(&found, &reply_context(identity, summary, now));
        RuleReplyText {
            subject: rendered.subject,
            body: rendered.body,
        }
    })?;
    Ok(())
}

/// Crée un brouillon par modèle demandé par les règles, sauf pour un message qui ne mérite pas de réponse
pub(super) fn create_reply_drafts(
    queue: &QueueContext<'_>,
    message: &MessageSummary,
    templates: &[TemplateId],
) -> Result<(), MiraError> {
    if !deserves_reply(message) {
        return Ok(());
    }
    for (rank, template) in templates.iter().enumerate() {
        prepare_reply(queue, message, rank, template)?;
    }
    Ok(())
}
