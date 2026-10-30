//! Réponse commune aux outils qui listent des messages : page rendue, ou erreur expliquée.

use super::render::mail_list;
use super::run_result::ToolRun;
use crate::api::Response;

/// Transforme le résultat d'une lecture de messages en résultat d'outil.
pub(super) fn messages_run(result: Result<Response, String>) -> ToolRun {
    match result {
        Ok(Response::Messages(page)) => mail_list(&page),
        Ok(_) => ToolRun::error("réponse inattendue du moteur".to_string()),
        Err(text) => ToolRun::error(text),
    }
}
