//! Test de connexion IMAP réel d'un compte : connexion, authentification, fermeture propre.

use mira_vocabulary::{AuthMethodKind, MiraError, ServerSettings};

use crate::auth::LoginSecret;
use crate::imap::{ImapClient, ImapCredential, ImapLogin};
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
