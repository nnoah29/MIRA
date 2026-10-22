//! Domaine « réglages des libellés » (spec R3 §4) : liste, réglages partiels, ordre.

mod list;
mod overview;
mod reorder;
mod set;

use mira_vocabulary::MiraError;

use crate::api::contract::{LabelSettingsRequest, LabelSettingsRequest as R, Response};
use crate::api::engine::Engine;

/// Résultat d'une requête du domaine
type Answer = Result<Response, MiraError>;

/// Route une requête du domaine réglages des libellés
pub(crate) async fn route(engine: &Engine, request: LabelSettingsRequest) -> Answer {
    match request {
        R::ListLabels => list::handle_list_labels(engine),
        R::ReorderLabels { order } => reorder::handle_reorder_labels(engine, &order),
        R::SetLabelSettings {
            label,
            hidden,
            hide_from_inbox,
            notifications,
            counter,
            color,
        } => {
            let change = set::Change {
                hidden,
                hide_from_inbox,
                notifications,
                counter,
                color,
            };
            set::handle_set_label_settings(engine, &label, &change)
        }
    }
}
