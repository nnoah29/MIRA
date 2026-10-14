//! Suites d'un enregistrement : départ, confirmation du moteur, échec.

use super::field::Field;
use super::save_plan::SavePlan;
use super::state::ComposeState;

impl ComposeState {
    /// Note l'envoi du plan : les champs envoyés ne sont plus signalés, les invalides le sont.
    pub fn begin_save(&mut self, plan: &SavePlan) {
        for (field, _) in &plan.sent {
            self.fields[field.index()].flagged = false;
        }
        for (field, _) in &plan.invalid {
            self.fields[field.index()].flagged = true;
        }
        if !plan.sent.is_empty() {
            self.saving = Some(plan.sent.clone());
        }
    }

    /// Le moteur a enregistré : les textes envoyés deviennent la référence, le reste (tapé depuis) reste à enregistrer.
    pub fn finish_save(&mut self) {
        for (field, text) in self.saving.take().unwrap_or_default() {
            self.fields[field.index()].baseline = text;
            if field == Field::Body {
                self.had_html = false;
            }
        }
        self.saved_once = true;
    }

    /// L'enregistrement a échoué : tout reste à enregistrer et la fermeture demandée est annulée.
    pub fn fail_save(&mut self) {
        self.saving = None;
        self.close_after_save = false;
    }

    /// Demande (ou annule) la fermeture dès que l'enregistrement aura réussi.
    pub fn set_close_after_save(&mut self, close: bool) {
        self.close_after_save = close;
    }

    /// Rend la demande de fermeture après enregistrement et l'efface.
    pub fn take_close_after_save(&mut self) -> bool {
        std::mem::take(&mut self.close_after_save)
    }
}
