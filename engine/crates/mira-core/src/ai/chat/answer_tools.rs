//! Réponse sur plusieurs tours : les résultats d'outils sont masqués par la table de la génération, qui sert aussi à restituer.

use mira_vocabulary::ToolCall;
use serde_json::Value;

use super::answer::Answer;

impl Answer {
    /// Masque le texte d'un résultat d'outil et étend la restitution aux repères qu'il crée.
    pub(super) fn mask_tool_text(&mut self, text: &str) -> String {
        let masked = self.masker.mask(text);
        self.restorer.extend(self.masker.take_new());
        masked
    }

    /// Restitue les valeurs d'origine dans un texte complet.
    pub(super) fn restore_text(&self, text: &str) -> String {
        self.restorer.restore_text(text)
    }

    /// Appel dont les textes d'arguments sont restitués, pour l'exécuter sur les vraies valeurs.
    pub(super) fn restored_call(&self, call: &ToolCall) -> ToolCall {
        let Ok(mut arguments) = serde_json::from_str::<Value>(&call.arguments) else {
            return call.clone();
        };
        self.restore_value(&mut arguments);
        ToolCall {
            arguments: arguments.to_string(),
            ..call.clone()
        }
    }

    /// Restitue chaque texte d'une valeur JSON, à tous les niveaux.
    fn restore_value(&self, value: &mut Value) {
        match value {
            Value::String(text) => *text = self.restore_text(text),
            Value::Array(items) => items.iter_mut().for_each(|item| self.restore_value(item)),
            Value::Object(fields) => fields
                .values_mut()
                .for_each(|item| self.restore_value(item)),
            _ => {}
        }
    }
}
