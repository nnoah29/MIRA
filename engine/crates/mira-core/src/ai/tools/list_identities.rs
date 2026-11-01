//! Outil `list_identities` : identités d'envoi de chaque compte (adresse, nom affiché, signature), pour écrire sous le bon nom.

use mira_vocabulary::{Account, ContentKind, Identity, ToolSpec};

use super::execute::execute_read;
use super::render::{cut, one_line};
use super::run_result::ToolRun;
use crate::api::{AccountRequest, Engine, IdentityRequest, Request, Response};

pub(super) const NAME: &str = "list_identities";

const SCHEMA: &str = r#"{"type":"object","properties":{}}"#;
const SIGNATURE_CHARS: usize = 300;

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Liste les identités d'envoi de l'utilisateur (identifiant, adresse, nom affiché, signature) : à consulter pour écrire sous son nom et choisir l'expéditeur.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Liste les comptes puis les identités de chacun.
pub(super) async fn run(engine: &Engine, _arguments: &str) -> ToolRun {
    match execute_read(engine, Request::Account(AccountRequest::ListAccounts)).await {
        Ok(Response::Accounts(accounts)) => identities_text(engine, &accounts).await,
        Ok(_) => ToolRun::error("réponse inattendue du moteur".to_string()),
        Err(failure) => ToolRun::error(failure.text()),
    }
}

/// Une ligne par identité, groupées par compte.
async fn identities_text(engine: &Engine, accounts: &[Account]) -> ToolRun {
    let mut lines = Vec::new();
    for account in accounts {
        let request = Request::Identity(IdentityRequest::ListIdentities {
            account: account.id.clone(),
        });
        match execute_read(engine, request).await {
            Ok(Response::Identities(list)) => lines.extend(list.iter().map(identity_line)),
            Ok(_) => return ToolRun::error("réponse inattendue du moteur".to_string()),
            Err(failure) => return ToolRun::error(failure.text()),
        }
    }
    let count = lines.len();
    let mut run = ToolRun::listed(lines.join("\n"), ContentKind::ToolResult, Vec::new());
    run.results = u32::try_from(count).unwrap_or(u32::MAX);
    run
}

/// Ligne d'une identité : identifiant, compte, adresse, nom, état par défaut et signature coupée.
fn identity_line(identity: &Identity) -> String {
    let signature = identity.signature.as_deref().map(one_line);
    format!(
        "identité id={} compte={} adresse={} nom={} par_défaut={} signature={}",
        identity.id.as_str(),
        identity.account.as_str(),
        identity.address.as_str(),
        one_line(&identity.display_name),
        if identity.is_default { "oui" } else { "non" },
        signature.map_or("(aucune)".to_string(), |text| cut(&text, SIGNATURE_CHARS)),
    )
}
