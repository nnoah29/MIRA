//! Capacités effectives et choix de stratégie (D-08, D-22, 10.14).

pub mod effective;
pub mod parse;
pub mod server_capability;
pub mod strategy;

pub use effective::EffectiveCapabilities;
pub use parse::parse_capabilities;
pub use server_capability::ServerCapability;
pub use strategy::{
    ChangeDetection, ExpungeMethod, MoveMethod, Realtime, choose_change_detection, choose_expunge,
    choose_move, choose_realtime,
};
