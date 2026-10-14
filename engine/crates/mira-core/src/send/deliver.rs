//! Remise du message au serveur SMTP, avec un rafraîchissement silencieux du jeton si le serveur le refuse (7.7).

use mira_vocabulary::error::codes::network;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::actions::ReplayContext;
use crate::actions::replay::errors::smtp_not_configured;
use crate::smtp::compose::BuiltMessage;
use crate::smtp::{OutgoingMessage, send_message};

/// Envoie le message ; un refus d'authentification déclenche un rafraîchissement du jeton puis un seul nouvel essai
pub(crate) async fn deliver(
    ctx: &ReplayContext<'_>,
    built: &BuiltMessage,
) -> Result<(), MiraError> {
    let outgoing = built.clone().into_outgoing();
    match attempt(ctx, &outgoing).await {
        Err(error) if error.category == ErrorCategory::Authentication => {
            ctx.credentials
                .renew_now(ctx.account)
                .await
                .map_err(renewal_failure)?;
            attempt(ctx, &outgoing).await
        }
        other => other,
    }
}

/// Un échec de rafraîchissement n'a rien envoyé : une coupure du service de jetons n'est pas une issue incertaine
fn renewal_failure(error: MiraError) -> MiraError {
    if error.code != network::CONNECTION_LOST.0 {
        return error;
    }
    let message = "service d'autorisation injoignable, jeton non rafraîchi";
    MiraError::new(ErrorCategory::Network, network::UNREACHABLE, message).retryable(true)
}

/// Une tentative SMTP avec les identifiants actuels du compte
async fn attempt(ctx: &ReplayContext<'_>, outgoing: &OutgoingMessage) -> Result<(), MiraError> {
    let endpoint = ctx.smtp.ok_or_else(smtp_not_configured)?;
    let login = ctx.credentials.smtp_login(ctx.account)?;
    send_message(endpoint, &login, outgoing).await
}
