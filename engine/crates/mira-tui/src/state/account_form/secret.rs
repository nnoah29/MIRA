//! Texte secret saisi : jamais affiché, jamais imprimé par `Debug`, vidé dès qu'il part dans une requête.

use mira_assembly::api::SecretInput;

const MAX_BYTES: usize = 4096;

/// Mot de passe en cours de saisie ; `Debug` n'en montre que le fait qu'il est masqué.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct SecretText(String);

impl SecretText {
    /// Ajoute un caractère imprimable ; un caractère de contrôle ou un excès de 4 Ko est refusé.
    pub fn push(&mut self, c: char) {
        if !c.is_control() && self.0.len() < MAX_BYTES {
            self.0.push(c);
        }
    }

    /// Retire le dernier caractère.
    pub fn pop(&mut self) {
        self.0.pop();
    }

    /// Nombre de caractères saisis.
    pub fn char_count(&self) -> usize {
        self.0.chars().count()
    }

    /// Remet la valeur au contrat du moteur et laisse ce champ vide.
    pub fn take_input(&mut self) -> SecretInput {
        SecretInput::new(std::mem::take(&mut self.0))
    }

    /// Écrase puis vide la valeur sans la transmettre.
    pub fn wipe(&mut self) {
        let length = self.0.len();
        self.0.replace_range(.., &"\0".repeat(length));
        self.0.clear();
    }
}

impl std::fmt::Debug for SecretText {
    /// Masque la valeur dans tout affichage de débogage.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "SecretText(***)")
    }
}
