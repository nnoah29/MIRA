//! Poignée de main : le premier échange d'une connexion, borné dans le temps et sans texte libre.

use mira_assembly::mira_vocabulary::{CONTRACT_VERSION, ContractVersion};
use mira_protocol::{
    Auth, Frame, HelloRefusal, HostState, PROTOCOL_VERSION, ProtocolVersion, check_hello,
    client_is_newer,
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::OwnedSemaphorePermit;

use super::Access;
use crate::server::ServerContext;
use crate::wire::{WireError, read_frame, write_frame};

/// Taille maximale d'un `hello` : un client non authentifié ne fait pas allouer plus.
pub const HELLO_MAX_BYTES: u32 = 8 * 1024;

/// Droits accordés à un client et la place qu'il occupe tant que sa connexion dure.
pub type Admitted = (Access, OwnedSemaphorePermit);

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

/// Place de client identifié, prise avant d'annoncer `hello_ok` ; `busy` si les trois places sont prises.
fn take_slot(ctx: &ServerContext) -> Result<OwnedSemaphorePermit, HelloRefusal> {
    ctx.client_slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| HelloRefusal::Busy)
}

/// Réponse à un `hello` bien formé, les droits accordés et la place prise : `hello_ok`, ou le refus.
fn answer(
    ctx: &ServerContext,
    protocol: ProtocolVersion,
    contract: ContractVersion,
    auth: &Auth,
) -> (Frame, Option<Admitted>) {
    let admitted = verdict(ctx, protocol, contract, auth)
        .and_then(|access| take_slot(ctx).map(|slot| (access, slot)));
    match admitted {
        Ok((access, slot)) => {
            let reply = Frame::HelloOk {
                protocol_version: PROTOCOL_VERSION,
                contract_version: CONTRACT_VERSION,
                host_state: announced(ctx, access),
            };
            (reply, Some((access, slot)))
        }
        Err(reason) => (Frame::HelloErr { reason }, None),
    }
}

/// Lit `hello` dans le délai et prépare la réponse (`malformed` pour un `hello` illisible) ; rien si la connexion doit se fermer sans réponse.
async fn read_hello<R: AsyncRead + Unpin>(
    reader: &mut R,
    ctx: &ServerContext,
) -> Option<(Frame, Option<Admitted>)> {
    let limit = ctx.max_frame.min(HELLO_MAX_BYTES);
    match tokio::time::timeout(ctx.hello_timeout, read_frame(reader, limit)).await {
        Ok(Ok(Some(Frame::Hello {
            protocol_version,
            contract_version,
            auth,
            ..
        }))) => Some(answer(ctx, protocol_version, contract_version, &auth)),
        Ok(Err(WireError::Malformed)) => {
            let reason = HelloRefusal::Malformed;
            Some((Frame::HelloErr { reason }, None))
        }
        _ => None,
    }
}

/// Répond `hello_ok` ou `hello_err` ; rend les droits et la place, ou rien si la connexion doit se fermer.
pub async fn handshake<R, W>(
    reader: &mut R,
    writer: &mut W,
    ctx: &ServerContext,
) -> Option<Admitted>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let (reply, admitted) = read_hello(reader, ctx).await?;
    let written = write_frame(writer, &reply, ctx.max_frame).await.is_ok();
    admitted.filter(|_| written)
}
