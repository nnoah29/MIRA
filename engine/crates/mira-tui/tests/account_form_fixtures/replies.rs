//! Réponses du moteur au formulaire et requête d'ajout émise.
#![allow(dead_code)]

use mira_assembly::api::contract::AccountPreparation;
use mira_assembly::api::{AccountRequest, Request, Response};
use mira_assembly::mira_vocabulary::{Account, MiraError, ProviderDescriptor};
use mira_tui::commands::{Command, Tag};
use mira_tui::update::{Msg, Reply};

use super::proposed_servers;

/// Réponse du moteur à la préparation suivie par ce jeton, avec ce fournisseur et les serveurs de test.
pub fn prepared_reply(token: u64, descriptor: ProviderDescriptor) -> Msg {
    let preparation = AccountPreparation {
        provider: descriptor,
        servers: proposed_servers(),
    };
    Msg::Reply(Reply {
        tag: Tag::Prepared(token),
        result: Ok(Response::AccountPreparation(preparation)),
    })
}

/// Réponse du moteur à l'ajout suivie par ce jeton.
pub fn added_reply(token: u64, result: Result<Account, MiraError>) -> Msg {
    Msg::Reply(Reply {
        tag: Tag::AccountAdded(token),
        result: result.map(Response::Account),
    })
}

/// Requête d'ajout contenue dans ces commandes.
pub fn added_command(commands: &[Command]) -> AccountRequest {
    let found = commands.iter().find_map(|command| match command {
        Command::Execute {
            request: Request::Account(request @ AccountRequest::AddAccount { .. }),
            ..
        } => Some(request.clone()),
        _ => None,
    });
    found.expect("aucun AddAccount émis")
}

