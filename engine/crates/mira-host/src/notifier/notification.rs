//! Une notification prête à afficher et le récepteur que l'hôte fournit pour l'afficher.

/// Notification composée par le moteur ; `message` désigne le message à ouvrir au clic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub sound: bool,
    pub message: Option<String>,
}

/// Affichage des notifications : D-Bus pour le démon, Dart pour le pont embarqué.
pub trait NotificationSink: Send + Sync {
    /// Affiche la notification ; ne bloque pas et n'échoue pas (un affichage perdu n'arrête rien).
    fn show(&self, notification: Notification);
}
