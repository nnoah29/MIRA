//! Protocole de trames entre un client MIRA et le démon : types, codec et règles de poignée de main.
//! Ce crate est pur : aucune entrée-sortie, aucun runtime ; le format écrit ici est la source de vérité des clients.

mod auth;
mod codec;
mod frame;
mod handshake;
mod version;

pub use auth::{Auth, HostOp, HostState};
pub use codec::{CodecError, MAX_FRAME_BYTES, body_len, decode, encode};
pub use frame::{EndReason, Frame};
pub use handshake::{HelloRefusal, check_hello};
pub use version::{PROTOCOL_VERSION, ProtocolVersion};
