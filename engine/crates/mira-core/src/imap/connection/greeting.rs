//! Bannière envoyée par le serveur à l'ouverture de la connexion.

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::imap::parse::capability::capabilities_from_code;
use crate::imap::parse::status::untagged_status;
use crate::imap::protocol::wire::Wire;

/// Ce que la bannière apprend : déjà authentifié ou non, capacités éventuellement annoncées.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Greeting {
    pub preauthenticated: bool,
    pub capabilities: Option<Vec<String>>,
}

/// Le serveur refuse la connexion dès la bannière.
fn refused() -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        server::REJECTED,
        "le serveur a refusé la connexion",
    )
}

/// Lit la bannière ; un `BYE` ou un texte inattendu est refusé.
pub(crate) async fn read_greeting(wire: &mut Wire) -> Result<Greeting, MiraError> {
    let line = wire.read_line().await?;
    let body = line.strip_prefix(b"* ").unwrap_or(&line);
    let Some((word, code, _)) = untagged_status(body) else {
        return Err(refused());
    };
    match word.as_str() {
        "OK" | "PREAUTH" => Ok(Greeting {
            preauthenticated: word == "PREAUTH",
            capabilities: capabilities_from_code(&code),
        }),
        _ => Err(refused()),
    }
}
