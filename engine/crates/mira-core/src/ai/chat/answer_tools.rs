//! Réponse sur plusieurs tours : les résultats d'outils sont masqués par la table de la génération, qui sert aussi à restituer.

use super::answer::Answer;

#[expect(dead_code, reason = "utilisé par la boucle d'outils")]
impl Answer {
    /// Masque le texte d'un résultat d'outil et étend la restitution aux repères qu'il crée.
    pub(super) fn mask_tool_text(&mut self, text: &str) -> String {
        let masked = self.masker.mask(text);
        self.restorer.extend(self.masker.take_new());
        masked
    }

    /// Restitue les valeurs d'origine dans un texte complet, par exemple les arguments d'un appel d'outil.
    pub(super) fn restore_text(&self, text: &str) -> String {
        self.restorer.restore_text(text)
    }
}
