//! Lecture des arguments de STORE.

use crate::imap::commands::args::text_arg;
use crate::imap::parser::command::Command;
use crate::imap::parser::token::Token;
use crate::imap::response::error::ImapError;
use crate::imap::store::entry_flags::FlagMode;

/// Opération STORE demandée.
#[derive(Debug, Clone)]
pub(crate) struct StoreSpec {
    pub mode: FlagMode,
    pub silent: bool,
    pub labels: bool,
    pub values: Vec<String>,
    pub unchanged_since: Option<u64>,
}

/// Lit modificateurs, opération et valeurs
pub(crate) fn parse_store(command: &Command) -> Result<StoreSpec, ImapError> {
    let modifiers = command.args.get(1).and_then(Token::list);
    let op_index = if modifiers.is_some() { 2 } else { 1 };
    let op = text_arg(command, op_index)?.to_ascii_uppercase();
    let (name, silent) = match op.strip_suffix(".SILENT") {
        Some(name) => (name.to_string(), true),
        None => (op.clone(), false),
    };
    let (mode, labels) = parse_op(&name)?;
    Ok(StoreSpec {
        mode,
        silent,
        labels,
        values: parse_values(command.args.get(op_index + 1))?,
        unchanged_since: parse_unchanged(modifiers)?,
    })
}

/// Mode et nature (drapeaux ou libellés) d'une opération
fn parse_op(name: &str) -> Result<(FlagMode, bool), ImapError> {
    match name {
        "+FLAGS" => Ok((FlagMode::Add, false)),
        "-FLAGS" => Ok((FlagMode::Remove, false)),
        "FLAGS" => Ok((FlagMode::Replace, false)),
        "+X-GM-LABELS" => Ok((FlagMode::Add, true)),
        "-X-GM-LABELS" => Ok((FlagMode::Remove, true)),
        "X-GM-LABELS" => Ok((FlagMode::Replace, true)),
        _ => Err(ImapError::bad("Unknown STORE operation")),
    }
}

/// Valeurs : liste parenthésée ou valeur unique
fn parse_values(token: Option<&Token>) -> Result<Vec<String>, ImapError> {
    match token {
        Some(Token::List(items)) => Ok(items.iter().filter_map(Token::text).collect()),
        Some(other) => Ok(other.text().into_iter().collect()),
        None => Err(ImapError::bad("Missing STORE values")),
    }
}

/// Valeur de UNCHANGEDSINCE dans la liste de modificateurs
fn parse_unchanged(modifiers: Option<&[Token]>) -> Result<Option<u64>, ImapError> {
    let items = modifiers.unwrap_or(&[]);
    let Some(position) = items
        .iter()
        .position(|t| t.keyword().as_deref() == Some("UNCHANGEDSINCE"))
    else {
        return Ok(None);
    };
    let value = items
        .get(position + 1)
        .and_then(Token::text)
        .and_then(|t| t.parse().ok());
    value
        .map(Some)
        .ok_or_else(|| ImapError::bad("Invalid UNCHANGEDSINCE"))
}
