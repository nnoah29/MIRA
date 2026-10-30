//! Outil `search_messages` : recherche plein texte locale avec filtres, vingt résultats par appel.

use mira_vocabulary::{MailboxId, ToolSpec};
use serde::Deserialize;

use super::args::{CommonArgs, filters_of, page_of, parse};
use super::execute::execute_read;
use super::messages::messages_run;
use super::run_result::ToolRun;
use crate::api::{Engine, Request, SearchRequest};
use crate::reading::SearchFilters;

pub(super) const NAME: &str = "search_messages";

const SCHEMA: &str = r#"{"type":"object","properties":{
"query":{"type":"string","description":"Mots à chercher dans les messages (sujet, expéditeur, corps)."},
"unread_only":{"type":"boolean"},"flagged_only":{"type":"boolean"},"has_attachments":{"type":"boolean"},
"from_contains":{"type":"string","description":"Texte contenu dans l'expéditeur."},
"to_contains":{"type":"string","description":"Texte contenu dans un destinataire."},
"mailbox":{"type":"string","description":"Identifiant de boîte, donné par list_mailboxes."},
"since":{"type":"string","description":"Date RFC 3339 ou AAAA-MM-JJ (incluse)."},
"until":{"type":"string","description":"Date RFC 3339 ou AAAA-MM-JJ (incluse)."},
"cursor":{"type":"string","description":"Curseur next_cursor d'un appel précédent, pour la suite."}
},"required":["query"]}"#;

/// Arguments de la recherche.
#[derive(Debug, Deserialize)]
struct SearchArgs {
    query: String,
    to_contains: Option<String>,
    mailbox: Option<String>,
    #[serde(flatten)]
    common: CommonArgs,
}

/// Fiche de l'outil annoncée au modèle.
pub(super) fn spec() -> ToolSpec {
    ToolSpec {
        name: NAME.to_string(),
        description: "Cherche des messages dans les boîtes de l'utilisateur et rend vingt lignes au plus (identifiant, expéditeur, date, sujet, aperçu).".to_string(),
        parameters_schema: SCHEMA.to_string(),
    }
}

/// Exécute la recherche demandée par le modèle.
pub(super) async fn run(engine: &Engine, arguments: &str) -> ToolRun {
    match request_of(arguments) {
        Ok(request) => messages_run(execute_read(engine, request).await),
        Err(text) => ToolRun::error(text),
    }
}

/// Requête `Search` tirée des arguments.
fn request_of(arguments: &str) -> Result<Request, String> {
    let args: SearchArgs = parse(arguments)?;
    let filters = SearchFilters {
        common: filters_of(&args.common)?,
        to_contains: args.to_contains.filter(|text| !text.is_empty()),
        mailbox: args
            .mailbox
            .filter(|id| !id.is_empty())
            .map(MailboxId::from_raw),
    };
    Ok(Request::Search(SearchRequest::Search {
        query: args.query,
        filters,
        page: page_of(args.common.cursor.as_ref()),
    }))
}
