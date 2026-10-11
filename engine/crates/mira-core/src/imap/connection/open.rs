//! Ouverture du flux selon le mode de chiffrement et lecture de la bannière.

use std::time::Duration;

use mira_vocabulary::{MiraError, Security, ServerEndpoint};

use super::capabilities::fetch_capabilities;
use super::greeting::{Greeting, read_greeting};
use super::starttls::upgrade_to_tls;
use super::tcp::connect_tcp;
use super::tls::wrap_tls;
use crate::imap::protocol::wire::{BoxedTransport, Wire};

/// Flux ouvert, bannière lue et capacités connues avant authentification.
#[derive(Debug)]
pub(crate) struct OpenedWire {
    pub wire: Wire,
    pub preauthenticated: bool,
    pub capabilities: Vec<String>,
}

/// Complète les capacités de la bannière en les demandant au serveur si besoin.
async fn known_capabilities(wire: &mut Wire, greeting: Greeting) -> Result<Vec<String>, MiraError> {
    match greeting.capabilities {
        Some(list) => Ok(list),
        None => fetch_capabilities(wire).await,
    }
}

/// Ouvre la connexion (clair, TLS ou STARTTLS) et lit la bannière du serveur.
pub(crate) async fn open_wire(
    endpoint: &ServerEndpoint,
    wait: Duration,
) -> Result<OpenedWire, MiraError> {
    let tcp = connect_tcp(endpoint, wait).await?;
    let transport: BoxedTransport = match endpoint.security {
        Security::Tls => wrap_tls(tcp, &endpoint.host).await?,
        _ => Box::new(tcp),
    };
    let mut wire = Wire::new(transport, wait);
    let greeting = read_greeting(&mut wire).await?;
    let preauthenticated = greeting.preauthenticated;
    let capabilities = if endpoint.security == Security::StartTls {
        wire = upgrade_to_tls(wire, &endpoint.host).await?;
        fetch_capabilities(&mut wire).await?
    } else {
        known_capabilities(&mut wire, greeting).await?
    };
    Ok(OpenedWire {
        wire,
        preauthenticated,
        capabilities,
    })
}
