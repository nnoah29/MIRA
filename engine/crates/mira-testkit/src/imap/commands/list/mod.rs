//! LIST (RFC 3501 section 6.3.8) avec SPECIAL-USE (RFC 6154).

mod attributes;
mod pattern;

use crate::imap::capabilities::Capability;
use crate::imap::parser::command::Command;
use crate::imap::parser::token::Token;
use crate::imap::response::error::HandlerResult;
use crate::imap::response::quoting::quote;
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;
use crate::imap::store::data::StoreData;

/// LIST : énumère les boîtes qui correspondent au motif
pub(crate) fn handle_list(state: &mut SessionState, command: &Command) -> HandlerResult {
    let user = state.require_user()?;
    let skip = usize::from(matches!(command.args.first(), Some(Token::List(_))));
    let words: Vec<String> = command
        .args
        .iter()
        .skip(skip)
        .filter_map(Token::text)
        .collect();
    let pattern = words.iter().take(2).cloned().collect::<String>();
    let special = state.ctx.caps.contains(Capability::SpecialUse);
    let lines = state
        .ctx
        .store
        .read(|data| list_lines(data, &user, &pattern, special));
    let mut reply = Reply::default();
    for line in lines {
        reply.untagged(&line);
    }
    Ok(reply.ok(&command.tag, "LIST completed"))
}

/// Lignes LIST des boîtes de l'utilisateur dont le nom correspond au motif
fn list_lines(data: &StoreData, user: &str, pattern: &str, special: bool) -> Vec<String> {
    if pattern.is_empty() {
        return vec!["LIST (\\Noselect) \"/\" \"\"".to_string()];
    }
    let Some(owner) = data.users.get(user) else {
        return Vec::new();
    };
    owner
        .mailboxes
        .iter()
        .filter(|(name, _)| pattern::glob_match(pattern, name))
        .map(|(name, mailbox)| {
            let attrs = attributes::attributes(owner, name, mailbox, special);
            format!("LIST ({attrs}) \"/\" {}", quote(name))
        })
        .collect()
}
