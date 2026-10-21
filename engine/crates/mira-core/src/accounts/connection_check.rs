//! Test de connexion IMAP réel d'un compte : connexion, authentification, fermeture propre.

use mira_vocabulary::{AuthMethodKind, MiraError, ServerSettings};

use crate::auth::LoginSecret;
use crate::imap::{ImapClient, ImapCredential, ImapLogin};
use crate::smtp::{SmtpCredential, SmtpLogin, check_smtp_login};
use crate::sync::ConnectionBudget;

/// Se connecte au serveur IMAP avec ces identifiants, dans le budget du compte, puis ferme la session
pub async fn check_imap_login(
    servers: &ServerSettings,
    login: &ImapLogin,
    budget: &ConnectionBudget,
) -> Result<(), MiraError> {
    let _permit = budget.acquire_explicit().await?;
    let client = ImapClient::connect(&servers.imap, login).await?;
    client.logout().await
}

/// Convertit le secret relu du coffre en identifiants IMAP (XOAUTH2 pour OAuth2)
pub fn imap_login_of(secret: LoginSecret) -> ImapLogin {
    let credential = match secret.method {
        AuthMethodKind::OAuth2 => ImapCredential::XOAuth2(secret.secret),
        AuthMethodKind::AppPassword | AuthMethodKind::Password => {
            ImapCredential::Password(secret.secret)
        }
    };
    ImapLogin {
        user: secret.user,
        credential,
    }
}

/// Convertit des identifiants IMAP en identifiants SMTP : le même utilisateur et le même secret
pub fn smtp_login_of(login: &ImapLogin) -> SmtpLogin {
    let credential = match &login.credential {
        ImapCredential::XOAuth2(token) => SmtpCredential::XOAuth2(token.clone()),
        ImapCredential::Password(secret) => SmtpCredential::Password(secret.clone()),
    };
    SmtpLogin {
        user: login.user.clone(),
        credential,
    }
}

/// Vérifie les deux serveurs du compte : IMAP dans le budget du compte, puis SMTP
pub async fn check_mail_login(
    servers: &ServerSettings,
    login: &ImapLogin,
    budget: &ConnectionBudget,
) -> Result<(), MiraError> {
    check_imap_login(servers, login, budget).await?;
    check_smtp_login(&servers.smtp, &smtp_login_of(login)).await
}
