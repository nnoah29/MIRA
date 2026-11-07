//! Relance de l'application MIRA depuis le démon (clic sur une notification, entrée du menu de l'icône).

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

const APP_ID: &str = "io.github.nnoah29.Mira";

/// Action qui lance l'app ; remplaçable dans les tests.
pub type Launch = Arc<dyn Fn() + Send + Sync>;

/// Commande à lancer : le binaire `mira` posé à côté du démon, sinon le lanceur du système.
pub fn launch_plan(exe_dir: Option<&Path>) -> (String, Vec<String>) {
    let sibling = exe_dir.map(|dir| dir.join("mira")).filter(|p| p.is_file());
    match sibling {
        Some(path) => (path.to_string_lossy().into_owned(), Vec::new()),
        None => ("gtk-launch".to_string(), vec![APP_ID.to_string()]),
    }
}

/// Lance l'app d'après le plan et laisse un fil récolter le processus fils.
fn spawn_app() -> std::io::Result<()> {
    let exe = std::env::current_exe().ok();
    let (program, arguments) = launch_plan(exe.as_deref().and_then(Path::parent));
    let mut child = Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

/// Lanceur de production : une erreur de lancement est ignorée, un clic perdu n'arrête rien.
pub fn app_launcher() -> Launch {
    Arc::new(|| {
        let _ = spawn_app();
    })
}
