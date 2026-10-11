//! Identifiants IMAP d'un compte à ajouter ou à tester, d'après le secret ou le flux OAuth fourni.

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{AccountId, AuthMethodKind, ErrorCategory, MiraError};

use super::draft_plan::DraftPlan;
use crate::accounts::imap_login_of;
use crate::api::contract::{CredentialInput, SecretInput};
use crate::api::engine::Engine;
use crate::auth::FlowTarget;
use crate::imap::{ImapCredential, ImapLogin};

/// Identifiants prêts pour le test de connexion, avec le compte qui les porte.
#[derive(Debug, Clone)]
pub(super) struct StagedCredential {
    pub account: AccountId,
    pub login: ImapLogin,
    pub flow: Option<String>,
}

/// Prépare les identifiants et range le mot de passe sous un nouvel identifiant de compte
pub(super) fn stage_credential(
    engine: &Engine,
    plan: &DraftPlan,
    credential: CredentialInput,
) -> Result<StagedCredential, MiraError> {
    let (CredentialInput::Password(input) | CredentialInput::AppPassword(input)) = credential
    else {
        return flow_credential(engine, plan, credential);
    };
    let account = AccountId::from_raw(engine.config.ids.next_id());
    let secret = require_filled(input)?;
    engine
        .vault()
        .store_password(&engine.config.profile, &account, &secret)?;
    let login = ImapLogin {
        user: plan.address.as_str().to_string(),
        credential: ImapCredential::Password(secret),
    };
    Ok(StagedCredential {
        account,
        login,
        flow: None,
    })
}

/// Prépare les identifiants d'un test de connexion sans rien ranger
pub(super) fn test_credential(
    engine: &Engine,
    plan: &DraftPlan,
    credential: CredentialInput,
) -> Result<ImapLogin, MiraError> {
    let (CredentialInput::Password(input) | CredentialInput::AppPassword(input)) = credential
    else {
        return Ok(flow_credential(engine, plan, credential)?.login);
    };
    Ok(ImapLogin {
        user: plan.address.as_str().to_string(),
        credential: ImapCredential::Password(require_filled(input)?),
    })
}

/// Relit au trousseau le jeton d'un flux OAuth terminé pour cette adresse et ce provider
fn flow_credential(
    engine: &Engine,
    plan: &DraftPlan,
    credential: CredentialInput,
) -> Result<StagedCredential, MiraError> {
    let CredentialInput::OAuthFlow { flow } = credential else {
        return Err(refusal("secret incompatible avec une autorisation OAuth"));
    };
    let summary = engine.oauth_flows.completed_summary(&flow)?;
    let target = &summary.target;
    require_flow_for_plan(plan, target)?;
    let user = plan.address.as_str();
    let secret = engine.vault().load_login(
        &engine.config.profile,
        &target.account,
        AuthMethodKind::OAuth2,
        user,
    )?;
    Ok(StagedCredential {
        account: target.account.clone(),
        login: imap_login_of(secret),
        flow: Some(flow),
    })
}

/// Refuse un flux qui concerne une autre adresse, un autre provider, ou une réauthentification
fn require_flow_for_plan(plan: &DraftPlan, target: &FlowTarget) -> Result<(), MiraError> {
    if target.address != plan.address || target.provider != plan.provider.descriptor().id {
        return Err(refusal(
            "le flux OAuth concerne une autre adresse ou un autre provider",
        ));
    }
    if target.reauthentication {
        return Err(refusal(
            "un flux de réauthentification ne peut pas créer de compte",
        ));
    }
    Ok(())
}

/// Refuse un secret vide
fn require_filled(input: SecretInput) -> Result<crate::ports::SecretValue, MiraError> {
    if input.is_empty() {
        return Err(refusal("le mot de passe ne peut pas être vide"));
    }
    Ok(input.into_secret_value())
}

/// Erreur de paramètre invalide
fn refusal(message: &str) -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        message,
    )
}
