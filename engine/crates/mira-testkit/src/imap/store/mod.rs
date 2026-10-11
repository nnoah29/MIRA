//! Modèle de données du faux serveur : utilisateurs, boîtes, messages partagés.

pub(crate) mod admin;
mod control;
pub(crate) mod data;
pub(crate) mod deliver;
pub(crate) mod entry_flags;
pub(crate) mod faults;
pub(crate) mod gmail_labels;
mod handle;
mod inspect;
pub(crate) mod labels;
pub(crate) mod mailbox;
mod mutate;
pub(crate) mod names;
mod renumber;
pub(crate) mod rows;
pub(crate) mod user;
pub(crate) mod view;

pub use handle::FakeMailStore;
pub use inspect::FakeMessageView;
