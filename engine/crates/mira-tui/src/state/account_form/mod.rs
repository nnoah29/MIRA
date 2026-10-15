//! Formulaire d'ajout de compte : fermé ou ouvert, avec un compteur de jetons qui ne revient jamais en arrière.

pub mod field;
pub mod form;
pub mod prepared;
pub mod secret;
pub mod visible;

pub use field::{FormField, Phase};
pub use form::AccountForm;
pub use prepared::Prepared;
pub use secret::SecretText;

/// Emplacement du formulaire ; les jetons survivent à sa fermeture pour qu'une réponse tardive ne soit jamais reprise.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AccountFormSlot {
    form: Option<AccountForm>,
    tokens: u64,
}

impl AccountFormSlot {
    /// Ouvre un formulaire vide.
    pub fn open(&mut self) {
        self.form = Some(AccountForm::new());
    }

    /// Ferme le formulaire après avoir effacé le mot de passe ; les réponses attendues seront ignorées.
    pub fn close(&mut self) {
        if let Some(form) = &mut self.form {
            form.password.wipe();
        }
        self.form = None;
    }

    /// Vrai si le formulaire est ouvert.
    pub fn is_open(&self) -> bool {
        self.form.is_some()
    }

    /// Formulaire ouvert.
    pub fn form(&self) -> Option<&AccountForm> {
        self.form.as_ref()
    }

    /// Formulaire ouvert, en écriture.
    pub fn form_mut(&mut self) -> Option<&mut AccountForm> {
        self.form.as_mut()
    }

    /// Jeton neuf, jamais rendu auparavant dans la session.
    pub fn next_token(&mut self) -> u64 {
        self.tokens += 1;
        self.tokens
    }
}
