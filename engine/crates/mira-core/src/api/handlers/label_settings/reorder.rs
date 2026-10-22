//! Ordre d'affichage des libellés dans la barre latérale.

use std::collections::HashSet;

use mira_vocabulary::{EventKind, LabelRef, MiraError};

use super::overview::load_overviews;
use crate::actions::errors::input::invalid_parameter;
use crate::api::contract::{LabelSettingsResponse, Response};
use crate::api::engine::Engine;
use crate::events::EventSink;

/// Fixe `position` selon la liste, qui doit nommer exactement tous les libellés existants, une fois chacun
pub(crate) fn handle_reorder_labels(
    engine: &Engine,
    order: &[LabelRef],
) -> Result<Response, MiraError> {
    engine.database.write(|tx| {
        let known: HashSet<LabelRef> = load_overviews(tx)?.into_iter().map(|i| i.label).collect();
        let given: HashSet<LabelRef> = order.iter().cloned().collect();
        if given != known || given.len() != order.len() {
            return Err(invalid_parameter(
                "l'ordre doit nommer chaque libellé une fois",
            ));
        }
        for (index, label) in order.iter().enumerate() {
            let mut settings = tx.get_label_settings(label)?;
            settings.position = u32::try_from(index).ok();
            tx.put_label_settings(label, &settings)?;
        }
        Ok(())
    })?;
    engine.events.emit(EventKind::LabelSettingsChanged)?;
    Ok(Response::LabelSettings(LabelSettingsResponse::Done))
}
