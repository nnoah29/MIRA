//! Ce que le moteur propose pour une adresse : fournisseur retenu, méthodes d'authentification et serveurs.

use mira_assembly::mira_vocabulary::{
    AuthMethodKind, ProviderDescriptor, ProviderId, ServerSettings,
};

/// Préparation reçue du moteur, réduite à ce que le formulaire affiche et renvoie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub provider: ProviderId,
    pub provider_name: String,
    pub manual_servers: bool,
    pub methods: Vec<AuthMethodKind>,
    pub unavailable: Vec<AuthMethodKind>,
    pub servers: ServerSettings,
}

/// Vrai pour une méthode que ce formulaire sait saisir dans cette version.
pub fn is_available(kind: AuthMethodKind) -> bool {
    match kind {
        AuthMethodKind::Password | AuthMethodKind::AppPassword => true,
        AuthMethodKind::OAuth2 => false,
    }
}

impl Prepared {
    /// Retient les méthodes saisissables et met à part les autres ; un fournisseur sans domaine propre est le repli.
    pub fn new(descriptor: ProviderDescriptor, servers: ServerSettings) -> Prepared {
        let (methods, unavailable) = descriptor
            .auth_methods
            .iter()
            .map(|method| method.kind())
            .partition(|kind| is_available(*kind));
        Prepared {
            provider: descriptor.id,
            provider_name: descriptor.display_name,
            manual_servers: descriptor.domains.is_empty(),
            methods,
            unavailable,
            servers,
        }
    }
}
