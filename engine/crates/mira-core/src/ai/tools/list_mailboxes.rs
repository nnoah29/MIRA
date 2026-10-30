//! Outil `list_mailboxes` : comptes et boîtes avec leurs identifiants et leurs compteurs de non lus.

use mira_vocabulary::{Account, ContentKind, Mailbox, ToolSpec};

use super::execute::execute_read;
use super::run_result::ToolRun;
use crate::api::{AccountRequest, Engine, MailboxRequest, Request, Response};

pub(super) const NAME: &str = "list_mailboxes";

const SCHEMA: &str = r#"{"type":"object","properties":{}}"#;

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Liste les comptes et leurs boîtes (identifiant, nom, messages non lus), pour filtrer par boîte ou compter les non lus.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Liste les comptes puis les boîtes de chacun.
pub(super) async fn run(engine: &Engine, _arguments: &str) -> ToolRun {
    match execute_read(engine, Request::Account(AccountRequest::ListAccounts)).await {
        Ok(Response::Accounts(accounts)) => accounts_text(engine, &accounts).await,
        Ok(_) => ToolRun::error("réponse inattendue du moteur".to_string()),
        Err(failure) => ToolRun::error(failure.text()),
    }
}

/// Une section par compte : sa ligne, puis une ligne par boîte.
async fn accounts_text(engine: &Engine, accounts: &[Account]) -> ToolRun {
    let mut lines = Vec::new();
    let mut count = 0;
    for account in accounts {
        lines.push(format!(
            "compte id={} adresse={}",
            account.id.as_str(),
            account.address.as_str()
        ));
        let request = Request::Mailbox(MailboxRequest::ListMailboxes {
            account: account.id.clone(),
        });
        match execute_read(engine, request).await {
            Ok(Response::Mailboxes(list)) => {
                count += list.len();
                lines.extend(list.iter().map(mailbox_line));
            }
            Ok(_) => return ToolRun::error("réponse inattendue du moteur".to_string()),
            Err(failure) => return ToolRun::error(failure.text()),
        }
    }
    let mut run = ToolRun::listed(lines.join("\n"), ContentKind::ToolResult, Vec::new());
    run.results = u32::try_from(count).unwrap_or(u32::MAX);
    run
}

/// Ligne d'une boîte : identifiant, nom affiché, non lus et total.
fn mailbox_line(mailbox: &Mailbox) -> String {
    format!(
        "boîte id={} nom={} non_lus={} total={}",
        mailbox.id.as_str(),
        mailbox.display_name.replace('\n', " "),
        mailbox.unread,
        mailbox.total
    )
}
