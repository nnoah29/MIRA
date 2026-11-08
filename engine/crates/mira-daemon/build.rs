//! Ressources Windows du démon : l'icône de la zone de notification.

/// Intègre l'icône au binaire quand la cible est Windows (`MIRA_SKIP_WIN_RESOURCES` l'ignore pour une compilation croisée de vérification).
fn main() {
    println!("cargo:rerun-if-changed=resources/mira-daemon.rc");
    let windows = std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows");
    let skipped = std::env::var_os("MIRA_SKIP_WIN_RESOURCES").is_some();
    if !windows || skipped {
        return;
    }
    let compiled = embed_resource::compile("resources/mira-daemon.rc", embed_resource::NONE);
    if compiled.manifest_required().is_err() {
        std::process::exit(1);
    }
}
