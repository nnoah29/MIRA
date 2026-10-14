//! Service de synchronisation du moteur : superviseur lancé à l'ouverture, piloté par les comptes et la façade.

mod accounts;
mod credentials;
mod requests;
mod service;

pub(crate) use requests::Completion;
pub(crate) use service::SyncService;
