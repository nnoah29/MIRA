//! Formulaire d'ajout de compte : champs saisis, étape, préparation reçue et erreur à montrer.

use mira_assembly::mira_vocabulary::{AuthMethodKind, IdempotencyKey, Security};

use super::field::{FormField, Phase};
use super::prepared::Prepared;
use super::secret::SecretText;
use crate::state::compose::TextBuffer;

/// Saisie en cours ; le mot de passe n'est jamais lisible ni imprimable (`SecretText`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountForm {
    pub address: TextBuffer,
    pub name: TextBuffer,
    pub imap_host: TextBuffer,
    pub imap_port: TextBuffer,
    pub smtp_host: TextBuffer,
    pub smtp_port: TextBuffer,
    pub imap_security: Security,
    pub smtp_security: Security,
    pub password: SecretText,
    pub focus: FormField,
    pub phase: Phase,
    pub prepared: Option<Prepared>,
    pub method: usize,
    pub error: Option<String>,
    pub confirm_close: bool,
    pub key: Option<IdempotencyKey>,
}

impl AccountForm {
    /// Formulaire vide, le focus sur l'adresse.
    pub fn new() -> AccountForm {
        let empty = || TextBuffer::single_line("");
        AccountForm {
            address: empty(),
            name: empty(),
            imap_host: empty(),
            imap_port: empty(),
            smtp_host: empty(),
            smtp_port: empty(),
            imap_security: Security::Tls,
            smtp_security: Security::Tls,
            password: SecretText::default(),
            focus: FormField::Address,
            phase: Phase::Typing,
            prepared: None,
            method: 0,
            error: None,
            confirm_close: false,
            key: None,
        }
    }

    /// Méthode d'authentification choisie parmi celles que le moteur propose et que ce formulaire sait saisir.
    pub fn method_kind(&self) -> Option<AuthMethodKind> {
        let prepared = self.prepared.as_ref()?;
        prepared.methods.get(self.method).copied()
    }

    /// Vrai dès qu'un champ est rempli ou que l'adresse a été préparée.
    pub fn has_input(&self) -> bool {
        self.prepared.is_some()
            || self.password.char_count() > 0
            || !self.address.text().is_empty()
            || !self.name.text().is_empty()
    }

    /// Tampon de texte du champ, s'il est de texte libre (ni méthode ni mot de passe).
    pub fn buffer_mut(&mut self, field: FormField) -> Option<&mut TextBuffer> {
        match field {
            FormField::Address => Some(&mut self.address),
            FormField::Name => Some(&mut self.name),
            FormField::ImapHost => Some(&mut self.imap_host),
            FormField::ImapPort => Some(&mut self.imap_port),
            FormField::SmtpHost => Some(&mut self.smtp_host),
            FormField::SmtpPort => Some(&mut self.smtp_port),
            FormField::Method | FormField::Password => None,
        }
    }
}

impl Default for AccountForm {
    /// Formulaire vide.
    fn default() -> AccountForm {
        AccountForm::new()
    }
}
