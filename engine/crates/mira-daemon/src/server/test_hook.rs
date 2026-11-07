//! Fonction que le démon appelle pour afficher une notification d'essai, posée une fois le bureau relié.

use std::sync::OnceLock;

/// Emplacement de la fonction d'essai ; vide tant que le bureau n'est pas relié.
#[derive(Default)]
pub struct TestHook {
    hook: OnceLock<Box<dyn Fn() + Send + Sync>>,
}

impl std::fmt::Debug for TestHook {
    /// Ne montre pas la fonction.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TestHook")
    }
}

impl TestHook {
    /// Pose la fonction d'essai ; la première posée reste.
    pub fn set(&self, hook: impl Fn() + Send + Sync + 'static) {
        let _ = self.hook.set(Box::new(hook));
    }

    /// Appelle la fonction d'essai si elle est posée.
    pub fn run(&self) {
        if let Some(hook) = self.hook.get() {
            hook();
        }
    }
}
