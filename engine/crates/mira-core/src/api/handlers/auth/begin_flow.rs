//! Démarrage commun des flux OAuth : nouvel ajout de compte ou réauthentification.

use mira_vocabulary::{AccountId, AuthMethodKind, EmailAddress, MiraError, ProviderDescriptor};
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::accounts::require_auth_method;
use crate::api::contract::{OAuthStart, Response};
use crate::api::engine::Engine;
use crate::auth::{FlowEntry, FlowTarget, start_flow};

/// Ouvre l'écouteur local, enregistre le flux sous le compte réservé et rend l'adresse d'autorisation
pub(super) async fn begin_flow(
    engine: &Engine,
    descriptor: &ProviderDescriptor,
    address: &EmailAddress,
    account: AccountId,
    reauthentication: bool,
) -> Result<Response, MiraError> {
    require_auth_method(descriptor, AuthMethodKind::OAuth2)?;
    let (settings, client_id) = engine.config.oauth.resolve(descriptor)?;
    let mut rng = StdRng::from_os_rng();
    let mut pending = start_flow(&settings, &client_id, address.as_str(), &mut rng).await?;
    pending.set_client_secret(engine.config.oauth.client_secret(descriptor));
    let authorize_url = pending.authorize_url().to_string();
    let flow = engine.config.ids.next_id();
    let target = FlowTarget {
        account,
        address: address.clone(),
        provider: descriptor.id.clone(),
        reauthentication,
    };
    engine
        .oauth_flows
        .register(flow.clone(), FlowEntry::new(pending, target));
    Ok(Response::OAuthStarted(OAuthStart {
        flow,
        authorize_url,
    }))
}
