//! Libellés locaux sous forme portable : nom et couleur en clair, sans identifiant ni pose.

mod color;
mod from_portable;
mod to_portable;

pub use color::{format_color, parse_color};
pub use from_portable::{PortableLabel, PortableLabelError, from_portable};
pub use to_portable::to_portable;
