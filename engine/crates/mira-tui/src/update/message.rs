//! Messages que la boucle fournit à `update` et leur routage.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{Event, MiraError};

use super::accounts_reply::on_accounts;
use super::action_reply::{on_undo, on_user_action};
use super::compose::created::on_draft_created;
use super::compose::deleted::on_draft_deleted;
use super::compose::saved::on_draft_saved;
use super::engine_event::on_event;
use super::intent::{Intent, on_intent};
use super::mailboxes_reply::on_mailboxes;
use super::messages_reply::on_messages;
use super::reader_reply::{on_action, on_attachments, on_body, on_thread};
use super::search_reply::on_search;
use super::status_reply::{on_start_sync, on_status, on_sync_status};
use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Réponse du moteur à une requête étiquetée.
#[derive(Debug)]
pub struct Reply {
    pub tag: Tag,
    pub result: Result<Response, MiraError>,
}

/// Tout ce qui peut faire évoluer l'état.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Msg {
    Intent(Intent),
    Reply(Reply),
    Event(Event),
    Resize(u16, u16),
}

/// Applique un message à l'état et rend les effets à réaliser.
pub fn update(state: &mut AppState, msg: Msg) -> Vec<Command> {
    match msg {
        Msg::Intent(intent) => on_intent(state, intent),
        Msg::Reply(reply) => on_reply(state, reply),
        Msg::Event(event) => on_event(state, event),
        Msg::Resize(width, height) => on_resize(state, width, height),
    }
}

/// Enregistre la nouvelle taille de l'écran.
fn on_resize(state: &mut AppState, width: u16, height: u16) -> Vec<Command> {
    state.width = width;
    state.height = height;
    Vec::new()
}

/// Traite la réponse du moteur selon son étiquette.
fn on_reply(state: &mut AppState, reply: Reply) -> Vec<Command> {
    match reply.tag {
        Tag::Accounts => on_accounts(state, reply.result),
        Tag::Mailboxes(account) => on_mailboxes(state, account, reply.result),
        Tag::Messages { view, after } => on_messages(state, view, after, reply.result),
        Tag::Body(id) => on_body(state, id, reply.result),
        Tag::Attachments(id) => on_attachments(state, id, reply.result),
        Tag::Thread(thread) => on_thread(state, thread, reply.result),
        Tag::Action => on_action(state, reply.result),
        Tag::UserAction(key) => on_user_action(state, key, reply.result),
        Tag::Undo(action) => on_undo(state, action, reply.result),
        Tag::Search(query) => on_search(state, query, reply.result),
        Tag::StartSync => on_start_sync(state, reply.result),
        Tag::SyncStatus => on_sync_status(state, reply.result),
        Tag::Status => on_status(state, reply.result),
        Tag::DraftCreated(token) => on_draft_created(state, token, reply.result),
        Tag::DraftSaved(draft) => on_draft_saved(state, draft, reply.result),
        Tag::DraftDeleted(draft) => on_draft_deleted(state, draft, reply.result),
    }
}
