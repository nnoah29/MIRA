//! FETCH et UID FETCH avec CHANGEDSINCE et VANISHED.

mod body;
mod item;
mod modifiers;
mod prepare;
mod render;
mod rows;

use super::args::text_arg;
use crate::imap::parser::command::Command;
use crate::imap::parser::fetch_items::{FetchItem, parse_fetch_items};
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// FETCH : renvoie les éléments demandés pour chaque message de l'ensemble
pub(crate) fn handle_fetch(state: &mut SessionState, command: &Command) -> HandlerResult {
    let (user, mailbox) = state.target()?;
    let read_only = state.require_selected()?.read_only;
    let set = text_arg(command, 0)?;
    let items_token = command
        .args
        .get(1)
        .ok_or_else(|| ImapError::bad("Missing fetch items"))?;
    let options = modifiers::parse_modifiers(command.args.get(2))?;
    let mut items = parse_fetch_items(items_token)?;
    prepare::prepare_items(state, command, &options, &mut items)?;
    let request = rows::FetchRequest {
        user,
        mailbox,
        set,
        by_uid: command.uid,
        mark_seen: !read_only && prepare::has_unpeeked_body(&items),
        labels: items.contains(&FetchItem::GmLabels),
        changed_since: options.changed_since,
    };
    let found = state
        .ctx
        .store
        .write(|data| rows::collect_rows(data, &request))?;
    let mut reply = Reply::default();
    modifiers::announce_vanished(state, &options, &mut reply)?;
    render::render_rows(&mut reply, &found, &items);
    Ok(reply.ok(&command.tag, "Fetch completed"))
}
