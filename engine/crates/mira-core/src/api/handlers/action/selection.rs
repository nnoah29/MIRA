//! Résolution des cibles d'une action en lot : liste donnée, ou toutes les pages d'une requête de liste.

use mira_vocabulary::{AccountId, MessageId, MiraError};

use crate::actions::errors::input::{invalid_parameter, missing_parameter};
use crate::api::contract::TargetSelection;
use crate::api::engine::Engine;
use crate::reading::{ListQuery, list_messages};

const MAX_SELECTION: usize = 10_000;

/// Rend les messages visés, sans doublon et dans l'ordre ; une sélection vide est refusée
pub(super) fn resolve_selection(
    engine: &Engine,
    targets: TargetSelection,
) -> Result<Vec<MessageId>, MiraError> {
    let messages = match targets {
        TargetSelection::Messages { messages } => messages,
        TargetSelection::Query { query } => collect_query(engine, query)?,
    };
    let mut unique: Vec<MessageId> = Vec::new();
    for message in messages {
        if !unique.contains(&message) {
            unique.push(message);
        }
    }
    if unique.is_empty() {
        return Err(missing_parameter("aucun message ciblé"));
    }
    Ok(unique)
}

/// Lit toutes les pages de la requête dans la base locale, avec un plafond de sécurité
fn collect_query(engine: &Engine, mut query: ListQuery) -> Result<Vec<MessageId>, MiraError> {
    let accounts: Vec<AccountId> = engine.database.read(|tx| {
        let all = tx.list_accounts()?;
        Ok(all.into_iter().map(|account| account.id).collect())
    })?;
    let mut found = Vec::new();
    loop {
        let page = list_messages(&engine.database, &accounts, &query)?;
        found.extend(page.items.into_iter().map(|summary| summary.id));
        if found.len() > MAX_SELECTION {
            return Err(invalid_parameter("la sélection dépasse 10000 messages"));
        }
        match page.next_cursor {
            Some(cursor) => query.page.cursor = Some(cursor),
            None => return Ok(found),
        }
    }
}
