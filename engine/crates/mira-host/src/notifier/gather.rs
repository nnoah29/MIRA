//! Lectures du notifieur dans le moteur : réglages, en-tête du message et adresse du compte, jamais le corps.

use mira_assembly::api::{
    AccountRequest, MessageRequest, Request, Response, SettingsRequest, SettingsResponse,
};
use mira_assembly::mira_vocabulary::{AccountId, EngineSettings, MessageSummary};

use super::Arrival;
use super::pending::Pending;
use crate::Host;

/// Exécute une requête sans garder le moteur : il reste libre de se fermer entre deux lectures.
async fn call(host: &Host, request: Request) -> Option<Response> {
    let engine = host.engine()?;
    engine.execute(request).await.ok()
}

/// Réglages en vigueur du moteur.
pub(super) async fn settings(host: &Host) -> Option<EngineSettings> {
    match call(host, Request::Settings(SettingsRequest::GetSettings)).await? {
        Response::Settings(SettingsResponse::Settings(settings)) => Some(settings),
        _ => None,
    }
}

/// Adresse du compte, pour le texte générique d'une notification sans aperçu.
async fn address_of(host: &Host, account: &AccountId) -> Option<String> {
    let Response::Accounts(accounts) =
        call(host, Request::Account(AccountRequest::ListAccounts)).await?
    else {
        return None;
    };
    let found = accounts.iter().find(|candidate| &candidate.id == account)?;
    Some(found.address.as_str().to_string())
}

/// Expéditeur affiché : le nom s'il existe, sinon l'adresse.
fn sender_of(summary: &MessageSummary) -> String {
    summary
        .from
        .first()
        .map(|entry| entry.name.clone().unwrap_or_else(|| entry.address.clone()))
        .unwrap_or_default()
}

/// En-tête d'une arrivée ; rien si le message ou le compte ne se lisent plus.
pub(super) async fn arrival(host: &Host, pending: &Pending) -> Option<Arrival> {
    let request = Request::Message(MessageRequest::GetMessage {
        message: pending.message.clone(),
    });
    let Response::Message(summary) = call(host, request).await? else {
        return None;
    };
    Some(Arrival {
        account: pending.account.as_str().to_string(),
        address: address_of(host, &pending.account).await?,
        message: pending.message.as_str().to_string(),
        sender: sender_of(&summary),
        subject: summary.subject.clone(),
    })
}
