//! Champs du formulaire d'ajout de compte et étapes de son déroulement.

/// Champ qui peut avoir le focus ; ceux qui sont visibles dépendent de la préparation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Address,
    Name,
    Method,
    ImapHost,
    ImapPort,
    SmtpHost,
    SmtpPort,
    Password,
}

/// Étape du formulaire ; les numéros sont les jetons des requêtes attendues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Typing,
    Preparing(u64),
    Ready,
    Adding(u64),
}

impl FormField {
    /// Nom du champ tel qu'affiché devant sa valeur.
    pub fn label(self) -> &'static str {
        match self {
            FormField::Address => "Adresse",
            FormField::Name => "Nom affiché",
            FormField::Method => "Méthode",
            FormField::ImapHost => "Serveur IMAP",
            FormField::ImapPort => "Port IMAP",
            FormField::SmtpHost => "Serveur SMTP",
            FormField::SmtpPort => "Port SMTP",
            FormField::Password => "Mot de passe",
        }
    }
}
