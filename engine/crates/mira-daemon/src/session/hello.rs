//! Poignée de main : le premier échange d'une connexion, borné dans le temps et sans texte libre.

use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ContractVersion};
use mira_protocol::{
    Auth, Frame, HelloRefusal, HostState, PROTOCOL_VERSION, ProtocolVersion, check_hello,
    client_is_newer,
};
use tokio::io::{AsyncRead, AsyncWrite};

use super::Access;
use crate::server::ServerContext;
use crate::wire::{WireError, read_frame, write_frame};

/// Taille maximale d'un `hello` : un client non authentifié ne fait pas allouer plus.
pub const HELLO_MAX_BYTES: u32 = 8 * 1024;

/// Jeton d'abord (un inconnu n'apprend rien), puis versions : un client plus récent que le moteur n'est pas refusé, il est limité à la sortie de la mise à jour.
fn verdict(
    ctx: &ServerContext,
    protocol: ProtocolVersion,
    contract: ContractVersion,
    auth: &Auth,
) -> Result<Access, HelloRefusal> {
    let Auth::LocalToken { token } = auth;
    if !ctx.token.matches(token) {
        return Err(HelloRefusal::Auth);
    }
    if protocol.major == PROTOCOL_VERSION.major && client_is_newer(CONTRACT_VERSION, contract) {
        return Ok(Access::UpdateOnly);
    }
    check_hello(protocol, contract, CONTRACT_VERSION).map(|()| Access::Full)
}

/// État annoncé : `update_required` pour un client plus récent, sinon celui de l'hôte.
fn announced(ctx: &ServerContext, access: Access) -> HostState {
    match access {
        Access::UpdateOnly => HostState::UpdateRequired,
        Access::Full => ctx.host_state(),
    }
}

/// Réponse à un `hello` bien formé et les droits accordés : `hello_ok`, ou le refus.
fn answer(
    ctx: &ServerContext,
    protocol: ProtocolVersion,
    contract: ContractVersion,
    auth: &Auth,
) -> (Frame, Option<Access>) {
    match verdict(ctx, protocol, contract, auth) {
        Ok(access) => {
            let reply = Frame::HelloOk {
                protocol_version: PROTOCOL_VERSION,
                contract_version: CONTRACT_VERSION,
                host_state: announced(ctx, access),
            };
            (reply, Some(access))
        }
        Err(reason) => (Frame::HelloErr { reason }, None),
    }
}

/// Lit `hello` dans le délai, répond `hello_ok` ou `hello_err` (`malformed` pour un `hello` illisible) ; rend les droits accordés, ou rien si la connexion doit se fermer.
pub async fn handshake<R, W>(reader: &mut R, writer: &mut W, ctx: &ServerContext) -> Option<Access>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let read = tokio::time::timeout(
        ctx.hello_timeout,
        read_frame(reader, ctx.max_frame.min(HELLO_MAX_BYTES)),
    )
    .await;
    let (reply, access) = match read {
        Ok(Ok(Some(Frame::Hello {
            protocol_version,
            contract_version,
            auth,
            ..
        }))) => answer(ctx, protocol_version, contract_version, &auth),
        Ok(Err(WireError::Malformed)) => (
            Frame::HelloErr {
                reason: HelloRefusal::Malformed,
            },
            None,
        ),
        _ => return None,
    };
    let written = write_frame(writer, &reply, ctx.max_frame).await.is_ok();
    access.filter(|_| written)
}
