//! Identifiants de connexion SMTP : mot de passe ou jeton XOAUTH2 (valeurs masquées).

use crate::ports::SecretValue;

/// Secret présenté au serveur ; `Debug` n'affiche jamais la valeur.
#[derive(Debug, Clone)]
pub enum SmtpCredential {
    Password(SecretValue),
    XOAuth2(SecretValue),
}

/// Utilisateur et secret de connexion SMTP.
#[derive(Debug, Clone)]
pub struct SmtpLogin {
    pub user: String,
    pub credential: SmtpCredential,
}
