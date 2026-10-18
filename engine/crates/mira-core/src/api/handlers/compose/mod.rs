//! Domaine « rédaction » : brouillons, pièces jointes, envoi, boîte d'envoi.

mod attach_file;
mod cancel_send;
mod create_draft;
mod delete_draft;
mod get_draft;
mod list_drafts;
mod list_outbox;
mod remove_attachment;
mod resolve_uncertain_send;
mod save_draft_copy;
mod send_draft;
mod update_draft;

use mira_vocabulary::MiraError;

use self::attach_file::handle_attach_file;
use self::cancel_send::handle_cancel_send;
use self::create_draft::handle_create_draft;
use self::delete_draft::handle_delete_draft;
use self::get_draft::handle_get_draft;
use self::list_drafts::handle_list_drafts;
use self::list_outbox::handle_list_outbox;
use self::remove_attachment::handle_detach;
use self::resolve_uncertain_send::handle_resolve_uncertain_send;
use self::save_draft_copy::handle_save_copy;
use self::send_draft::handle_send_draft;
use self::update_draft::handle_update_draft;
use crate::api::contract::{ComposeRequest as Req, Response};
use crate::api::engine::Engine;

/// Route une requête du domaine rédaction
pub(crate) async fn route(engine: &Engine, request: Req) -> Result<Response, MiraError> {
    match request {
        Req::CreateDraft {
            account,
            identity,
            mode,
        } => handle_create_draft(engine, &account, identity.as_ref(), mode),
        Req::UpdateDraft { draft, changes } => handle_update_draft(engine, &draft, changes),
        Req::AttachFile { draft, path } => handle_attach_file(engine, &draft, &path),
        Req::RemoveAttachment { draft, position } => handle_detach(engine, &draft, position),
        Req::DeleteDraft { draft } => handle_delete_draft(engine, &draft),
        Req::SaveDraftCopy {
            draft,
            idempotency_key: k,
        } => handle_save_copy(engine, &draft, k),
        Req::GetDraft { draft } => handle_get_draft(engine, &draft),
        Req::ListDrafts { account } => handle_list_drafts(engine, &account),
        Req::SendDraft {
            draft,
            at,
            idempotency_key,
        } => handle_send_draft(engine, &draft, at, idempotency_key),
        Req::CancelSend { action } => handle_cancel_send(engine, &action),
        Req::ListOutbox { account } => handle_list_outbox(engine, &account),
        Req::ResolveUncertainSend { action, resend } => {
            handle_resolve_uncertain_send(engine, &action, resend)
        }
    }
}
