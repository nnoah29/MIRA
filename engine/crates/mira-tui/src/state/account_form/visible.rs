//! Champs visibles du formulaire selon la préparation, et passage de l'un à l'autre.

use super::field::{FormField, Phase};
use super::form::AccountForm;
use super::prepared::Prepared;
use crate::state::compose::TextBuffer;

impl AccountForm {
    /// Champs affichés, dans l'ordre du focus : l'adresse seule tant que rien n'est proposé.
    pub fn visible_fields(&self) -> Vec<FormField> {
        let mut fields = vec![FormField::Address];
        let Some(prepared) = &self.prepared else {
            return fields;
        };
        if prepared.methods.is_empty() {
            return fields;
        }
        fields.push(FormField::Name);
        if prepared.methods.len() > 1 {
            fields.push(FormField::Method);
        }
        if prepared.manual_servers {
            fields.extend([
                FormField::ImapHost,
                FormField::ImapPort,
                FormField::SmtpHost,
                FormField::SmtpPort,
            ]);
        }
        fields.push(FormField::Password);
        fields
    }

    /// Déplace le focus sur le champ visible suivant ou précédent, sans boucler.
    pub fn step_focus(&mut self, forward: bool) {
        let fields = self.visible_fields();
        let current = fields.iter().position(|field| *field == self.focus);
        let target = match (current, forward) {
            (Some(index), true) => index + 1,
            (Some(index), false) => index.saturating_sub(1),
            (None, _) => 0,
        };
        self.focus = fields.get(target).copied().unwrap_or(self.focus);
    }

    /// Tampon de texte du champ, s'il est de texte libre.
    pub fn buffer(&self, field: FormField) -> Option<&TextBuffer> {
        match field {
            FormField::Address => Some(&self.address),
            FormField::Name => Some(&self.name),
            FormField::ImapHost => Some(&self.imap_host),
            FormField::ImapPort => Some(&self.imap_port),
            FormField::SmtpHost => Some(&self.smtp_host),
            FormField::SmtpPort => Some(&self.smtp_port),
            FormField::Method | FormField::Password => None,
        }
    }

    /// Retient la préparation, préremplit les serveurs et place le focus sur le premier champ à compléter.
    pub fn apply_preparation(&mut self, prepared: Prepared) {
        let (imap, smtp) = (&prepared.servers.imap, &prepared.servers.smtp);
        self.imap_host = TextBuffer::single_line(&imap.host);
        self.imap_port = TextBuffer::single_line(&imap.port.to_string());
        self.smtp_host = TextBuffer::single_line(&smtp.host);
        self.smtp_port = TextBuffer::single_line(&smtp.port.to_string());
        self.imap_security = imap.security;
        self.smtp_security = smtp.security;
        self.focus = match (prepared.methods.is_empty(), prepared.manual_servers) {
            (true, _) => FormField::Address,
            (false, true) => FormField::ImapHost,
            (false, false) => FormField::Password,
        };
        self.method = 0;
        self.phase = Phase::Ready;
        self.error = None;
        self.prepared = Some(prepared);
    }

    /// Oublie la préparation (l'adresse a changé) : le mot de passe est effacé et une nouvelle clé sera tirée.
    pub fn reset_preparation(&mut self) {
        self.password.wipe();
        self.prepared = None;
        self.phase = Phase::Typing;
        self.key = None;
        self.method = 0;
        self.error = None;
    }
}
