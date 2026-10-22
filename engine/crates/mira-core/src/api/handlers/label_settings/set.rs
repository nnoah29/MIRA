//! Réglages partiels d'un libellé : seuls les champs fournis changent.

use mira_vocabulary::{CounterMode, EventKind, LabelRef, LabelSettings, MiraError};

use super::overview::{label_not_found, load_overviews};
use crate::api::contract::{LabelOverview, LabelSettingsResponse, Response};
use crate::api::engine::Engine;
use crate::events::EventSink;
use crate::labels::refresh_inbox_counters;
use crate::storage::Tx;

/// Champs à modifier ; un champ absent reste inchangé.
pub(super) struct Change {
    pub hidden: Option<bool>,
    pub hide_from_inbox: Option<bool>,
    pub notifications: Option<bool>,
    pub counter: Option<CounterMode>,
    pub color: Option<u32>,
}

/// Réglages après application des champs fournis
fn merged(before: &LabelSettings, change: &Change) -> LabelSettings {
    LabelSettings {
        hidden: change.hidden.unwrap_or(before.hidden),
        hide_from_inbox: change.hide_from_inbox.unwrap_or(before.hide_from_inbox),
        notifications: change.notifications.unwrap_or(before.notifications),
        counter: change.counter.unwrap_or(before.counter),
        color: change.color.or(before.color),
        position: before.position,
    }
}

/// Enregistre les réglages et rend la vue du libellé avec un indicateur « quelque chose a changé »
fn apply(
    tx: &Tx<'_>,
    label: &LabelRef,
    change: &Change,
) -> Result<(LabelOverview, bool), MiraError> {
    if !load_overviews(tx)?.iter().any(|item| &item.label == label) {
        return Err(label_not_found());
    }
    let before = tx.get_label_settings(label)?;
    let after = merged(&before, change);
    tx.put_label_settings(label, &after)?;
    let shown = load_overviews(tx)?
        .into_iter()
        .find(|item| &item.label == label);
    Ok((shown.ok_or_else(label_not_found)?, before != after))
}

/// Modifie les réglages du libellé ; inconnu : `not_found.label` ; annonce seulement si quelque chose change
pub(crate) fn handle_set_label_settings(
    engine: &Engine,
    label: &LabelRef,
    change: &Change,
) -> Result<Response, MiraError> {
    let (shown, changed) = engine.database.write(|tx| apply(tx, label, change))?;
    if changed {
        engine.events.emit(EventKind::LabelSettingsChanged)?;
        refresh_inbox_counters(&engine.database, &engine.events, true)?;
    }
    Ok(Response::LabelSettings(LabelSettingsResponse::Label(shown)))
}
