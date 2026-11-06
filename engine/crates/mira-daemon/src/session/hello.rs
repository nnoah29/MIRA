//! Poignée de main : le premier échange d'une connexion, borné dans le temps et sans texte libre.

use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ContractVersion};
use mira_protocol::{Auth, Frame, HelloRefusal, PROTOCOL_VERSION, ProtocolVersion, check_hello};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::server::ServerContext;
use crate::wire::{WireError, read_frame, write_frame};

/// Taille maximale d'un `hello` : un client non authentifié ne fait pas allouer plus.
pub const HELLO_MAX_BYTES: u32 = 8 * 1024;

/// Jeton d'abord (un inconnu n'apprend rien), puis versions.
fn verdict(
    ctx: &ServerContext,
    protocol: ProtocolVersion,
    contract: ContractVersion,
    auth: &Auth,
) -> Result<(), HelloRefusal> {
    let Auth::LocalToken { token } = auth;
    if !ctx.token.matches(token) {
        return Err(HelloRefusal::Auth);
    }
    check_hello(protocol, contract, CONTRACT_VERSION)
}

/// Réponse à un `hello` bien formé : `hello_ok` annonçant l'état, ou le refus.
fn answer(
    ctx: &ServerContext,
    protocol: ProtocolVersion,
    contract: ContractVersion,
    auth: &Auth,
) -> Frame {
    match verdict(ctx, protocol, contract, auth) {
        Ok(()) => Frame::HelloOk {
            protocol_version: PROTOCOL_VERSION,
            contract_version: CONTRACT_VERSION,
            host_state: ctx.host_state(),
        },
        Err(reason) => Frame::HelloErr { reason },
    }
}

/// Lit `hello` dans le délai, répond `hello_ok` ou `hello_err` (`malformed` pour un `hello` illisible) ; faux si la connexion doit se fermer.
pub async fn handshake<R, W>(reader: &mut R, writer: &mut W, ctx: &ServerContext) -> bool
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let read = tokio::time::timeout(
        ctx.hello_timeout,
        read_frame(reader, ctx.max_frame.min(HELLO_MAX_BYTES)),
    )
    .await;
    let reply = match read {
        Ok(Ok(Some(Frame::Hello {
            protocol_version,
            contract_version,
            auth,
            ..
        }))) => answer(ctx, protocol_version, contract_version, &auth),
        Ok(Err(WireError::Malformed)) => Frame::HelloErr {
            reason: HelloRefusal::Malformed,
        },
        _ => return false,
    };
    let accepted = matches!(reply, Frame::HelloOk { .. });
    write_frame(writer, &reply, ctx.max_frame).await.is_ok() && accepted
}
