//! Transport HTTP vers les services d'IA : HTTPS hors boucle locale, aucune redirection, délais et plafonds.
//! La clé est ajoutée au dernier moment ; aucun en-tête ni corps n'est journalisé ou recopié dans une erreur.

mod auth_header;
mod body;
mod client;
mod failure;
mod limits;
mod retry_after;
mod stream;

pub use body::get_bounded;
pub use failure::map_status;
pub use limits::TransportLimits;
pub use stream::{AiStreamBody, post_stream};
