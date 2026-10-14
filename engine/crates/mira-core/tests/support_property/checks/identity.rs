//! Invariants 7 et 10 : identifiant interne stable, un message = une ligne, un seul compte, correspondances serveur.

use std::collections::HashSet;

use mira_core::storage::repositories::server_ids::ServerIdKind;

use crate::support_property::store_world::{StoreWorld, account_id};

/// Invariant 7 : chaque message attendu existe sous son identifiant d'origine, dans son compte, et son identifiant stable y mène
pub fn check_internal_ids(world: &StoreWorld) {
    world
        .db
        .read(|tx| {
            for message in &world.model.messages {
                let stored = tx.get_message(&message.id)?.unwrap();
                assert_eq!(stored.account, account_id(message.account));
                let mapped = tx.find_by_server_id(
                    &account_id(message.account),
                    ServerIdKind::Stable,
                    &message.stable,
                )?;
                assert_eq!(mapped.as_ref(), Some(&message.id), "correspondance serveur");
            }
            Ok(())
        })
        .unwrap();
}

/// Un message = une ligne : autant de lignes et de correspondances que de messages, identifiants tous distincts
pub fn check_one_row_per_message(world: &StoreWorld) {
    let expected = i64::try_from(world.model.messages.len()).unwrap();
    let (rows, mappings) = world
        .db
        .read(|tx| {
            Ok((
                tx.table_row_count("messages")?,
                tx.table_row_count("server_ids")?,
            ))
        })
        .unwrap();
    assert_eq!(rows, expected, "lignes de messages");
    assert_eq!(mappings, expected, "correspondances serveur");
    let distinct: HashSet<&str> = world
        .model
        .messages
        .iter()
        .map(|message| message.id.as_str())
        .collect();
    assert_eq!(distinct.len(), world.model.messages.len());
}

/// Invariant 10 : la même valeur d'identifiant stable dans deux comptes mène à deux messages distincts, chacun dans son compte
pub fn check_ownership(world: &StoreWorld) {
    world
        .db
        .read(|tx| {
            for message in &world.model.messages {
                for other in 0..crate::support_property::model::ACCOUNTS {
                    let found = tx.find_by_server_id(
                        &account_id(other),
                        ServerIdKind::Stable,
                        &message.stable,
                    )?;
                    let mine = other == message.account;
                    assert!(!mine || found.as_ref() == Some(&message.id));
                    assert!(mine || found.as_ref() != Some(&message.id));
                }
            }
            Ok(())
        })
        .unwrap();
}
