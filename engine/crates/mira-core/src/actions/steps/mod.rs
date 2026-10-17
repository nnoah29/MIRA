//! Étapes élémentaires du miroir : application, inverse et reconnaissance d'un effet déjà en place.

mod appearances;
mod appearances_inverse;
mod dispatch;
mod flags;
mod flags_inverse;
mod mailboxes;
mod mailboxes_inverse;

pub(crate) use dispatch::{apply_step, capture_inverse, step_in_effect};
