//! Progression sur plusieurs tours : clôture d'un tour, appels et texte du tour pour l'historique.

use mira_vocabulary::ToolCall;

use super::progress::Progress;

impl Progress {
    /// Clôt un tour : sa consommation s'ajoute au total, raison d'arrêt et texte vu repartent de zéro.
    pub(super) fn close_round(&mut self) {
        if let Some((input, output)) = self.usage.take() {
            let (before_input, before_output) = self.spent.unwrap_or_default();
            self.spent = Some((
                before_input.saturating_add(input),
                before_output.saturating_add(output),
            ));
        }
        self.stop = None;
        self.text_seen = false;
        self.separate = self.emitted;
    }

    /// Appels complets du tour qui vient de finir.
    pub(super) fn take_calls(&mut self) -> Vec<ToolCall> {
        self.tools.finish()
    }

    /// Texte du tour tel que le modèle l'a écrit (repères compris), pour le renvoyer dans l'historique.
    pub(super) fn take_turn_text(&mut self) -> String {
        std::mem::take(&mut self.turn_text)
    }
}
