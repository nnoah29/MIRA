//! Outil `list_messages` : liste d'une vue (tous, non lus, suivis, une boîte), vingt résultats par appel.

use mira_vocabulary::{MailboxId, StandardView, ToolSpec, ViewRef};
use serde::Deserialize;

use super::args::{CommonArgs, filters_of, page_of, parse};
use super::execute::execute_read;
use super::messages::messages_run;
use super::run_result::ToolRun;
use crate::api::{Engine, MessageRequest, Request};
use crate::reading::ListQuery;

pub(super) const NAME: &str = "list_messages";

const SCHEMA: &str = r#"{"type":"object","properties":{
"view":{"type":"string","enum":["unified","unread","flagged","mailbox"],"description":"Vue à lister ; unified par défaut."},
"mailbox":{"type":"string","description":"Identifiant de boîte (obligatoire si view vaut mailbox), donné par list_mailboxes."},
"unread_only":{"type":"boolean"},"flagged_only":{"type":"boolean"},"has_attachments":{"type":"boolean"},
"from_contains":{"type":"string","description":"Texte contenu dans l'expéditeur."},
"since":{"type":"string","description":"Date RFC 3339 ou AAAA-MM-JJ (incluse)."},
"until":{"type":"string","description":"Date RFC 3339 ou AAAA-MM-JJ (incluse)."},
"cursor":{"type":"string","description":"Curseur next_cursor d'un appel précédent, pour la suite."}
}}"#;

/// Arguments de la liste.
#[derive(Debug, Deserialize)]
struct ListArgs {
    view: Option<String>,
    mailbox: Option<String>,
    #[serde(flatten)]
    common: CommonArgs,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Liste les messages récents d'une vue (tous, non lus, suivis ou une boîte), vingt lignes au plus.".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Exécute la liste demandée par le modèle.
pub(super) async fn run(engine: &Engine, arguments: &str) -> ToolRun {
    match request_of(arguments) {
        Ok(request) => messages_run(execute_read(engine, request).await),
        Err(text) => ToolRun::error(text),
    }
}

/// Requête `ListMessages` tirée des arguments.
fn request_of(arguments: &str) -> Result<Request, String> {
    let args: ListArgs = parse(arguments)?;
    let query = ListQuery {
        view: view_of(&args)?,
        filters: filters_of(&args.common)?,
        page: page_of(args.common.cursor.as_ref()),
    };
    Ok(Request::Message(MessageRequest::ListMessages(query)))
}

/// Vue demandée ; une boîte exige son identifiant, une vue inconnue est refusée.
fn view_of(args: &ListArgs) -> Result<ViewRef, String> {
    let standard = |view| Ok(ViewRef::Standard { view });
    match args.view.as_deref().unwrap_or("unified") {
        "unified" => standard(StandardView::Unified),
        "unread" => standard(StandardView::Unread),
        "flagged" => standard(StandardView::Flagged),
        "mailbox" => match args.mailbox.as_deref().filter(|id| !id.is_empty()) {
            Some(id) => Ok(ViewRef::Mailbox {
                mailbox: MailboxId::from_raw(id),
            }),
            None => Err("la vue mailbox demande l'argument mailbox".to_string()),
        },
        other => Err(format!(
            "vue inconnue « {} »",
            super::render::cut(other, 40)
        )),
    }
}
