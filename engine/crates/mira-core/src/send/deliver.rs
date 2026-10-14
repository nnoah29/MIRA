//! Remise du message au serveur SMTP, avec un rafraîchissement silencieux du jeton si le serveur le refuse (7.7).

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
            ctx.credentials.renew_now(ctx.account).await?;
            attempt(ctx, &outgoing).await
        }
        other => other,
    }
}

/// Une tentative SMTP avec les identifiants actuels du compte
async fn attempt(ctx: &ReplayContext<'_>, outgoing: &OutgoingMessage) -> Result<(), MiraError> {
    let endpoint = ctx.smtp.ok_or_else(smtp_not_configured)?;
    let login = ctx.credentials.smtp_login(ctx.account)?;
    send_message(endpoint, &login, outgoing).await
}
