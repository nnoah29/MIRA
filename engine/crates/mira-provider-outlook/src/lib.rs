//! Provider outlook : profil de données et particularités, sans entrée-sortie (D-05, D-07).
//! Les valeurs viennent du §6.7 de la référence et restent à vérifier auprès du service.

pub mod descriptor;
pub mod provider;
pub mod roles;
pub mod servers;

pub use provider::OutlookProvider;
