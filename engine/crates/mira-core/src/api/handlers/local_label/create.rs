//! Création d'un libellé local : nom contrôlé, plafond, couleur par défaut, idempotente par clé.

use mira_vocabulary::{IdempotencyKey, LocalLabel, LocalLabelId, MiraError};

use super::checks::{clean_name, ensure_room};
use super::idempotency::LabelCommand;
use super::writing::{announce, default_color, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;

/// Crée le libellé sous un identifiant du moteur ; la même clé rejouée rend le même libellé sans rien écrire
pub(crate) fn handle_create_local_label(
    engine: &Engine,
    name: &str,
    color: Option<u32>,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let name = clean_name(name)?;
    let command = LabelCommand::new(key, "create_local_label", &(&name, color))?;
    let id = LocalLabelId::from_raw(engine.config.ids.next_id());
    let (label, written) = engine.database.write(|tx| {
        if let Some(label) = command.replay::<LocalLabel>(tx)? {
            return Ok((label, false));
        }
        let existing = ensure_room(tx, 1)?;
        let color = color.unwrap_or_else(|| default_color(existing));
        let label = LocalLabel {
            id: id.clone(),
            name: name.clone(),
            color,
        };
        tx.insert_local_label(&label)?;
        command.record(tx, &label)?;
        Ok((label, true))
    })?;
    if written {
        announce(engine)?;
    }
    Ok(single(label))
}
