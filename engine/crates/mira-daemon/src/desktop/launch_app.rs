//! Relance de l'application MIRA depuis le démon (clic sur une notification, entrée du menu de l'icône).

use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;

const APP_ID: &str = "com.toffanoah.mira";
const APP_BIN: &str = if cfg!(windows) { "mira.exe" } else { "mira" };

/// Action qui lance l'app ; remplaçable dans les tests.
pub type Launch = Arc<dyn Fn() + Send + Sync>;

/// Commande à lancer : `MIRA_APP_BIN` s'il désigne un fichier, sinon le binaire `mira` posé à côté du démon, sinon le lanceur du système.
pub fn launch_plan(exe_dir: Option<&Path>, explicit: Option<&Path>) -> (String, Vec<String>) {
    let neighbour = exe_dir.map(|dir| dir.join(APP_BIN));
    let chosen = explicit.map(Path::to_path_buf).or(neighbour);
    match chosen.filter(|path| path.is_file()) {
        Some(path) => (path.to_string_lossy().into_owned(), Vec::new()),
        None => ("gtk-launch".to_string(), vec![APP_ID.to_string()]),
    }
}

/// Lance l'app d'après le plan et laisse un fil récolter le processus fils.
fn spawn_app() -> std::io::Result<()> {
    let exe = std::env::current_exe().ok();
    let explicit = std::env::var_os("MIRA_APP_BIN").map(std::path::PathBuf::from);
    let exe_dir = exe.as_deref().and_then(Path::parent);
    let (program, arguments) = launch_plan(exe_dir, explicit.as_deref());
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
