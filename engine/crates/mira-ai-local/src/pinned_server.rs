//! Archives du serveur épinglées par tag, taille et somme SHA-256.

use mira_vocabulary::AiLocalServerBuild;

const SERVER_TAG_URL: &str = "https://github.com/ggml-org/llama.cpp/releases/download/b12224";

/// Archives du serveur pour les plateformes prises en charge.
pub(crate) fn server_builds() -> Vec<AiLocalServerBuild> {
    vec![
        build(
            "linux-x86_64",
            "llama-b12224-bin-ubuntu-x64.tar.gz",
            23_109_432,
            "e3f9e11e61b8bcecccc4b7522d131cc6d3ec18fc3f31f6caa58ddda63801bbd5",
        ),
        build(
            "linux-aarch64",
            "llama-b12224-bin-ubuntu-arm64.tar.gz",
            18_759_680,
            "9e66367e598fb0c7a196e3b6586b9c0b455fec0f5364b962a05ffd53aebc14be",
        ),
        build(
            "windows-x86_64",
            "llama-b12224-bin-win-cpu-x64.zip",
            24_887_408,
            "d7a7a3e014a02dbfce46977f9da6d9b58196b9c83b80bcf6ef2337134103c3d6",
        ),
        build(
            "windows-aarch64",
            "llama-b12224-bin-win-cpu-arm64.zip",
            13_237_558,
            "7d3cb8f7076337c51b5182ad45cfc2b12f0f2ed30efa66196f2f6f0b68a6c23d",
        ),
    ]
}

/// Une archive du serveur ; l'exécutable est cherché dans le dossier d'extraction.
fn build(platform: &str, archive: &str, size: u64, sha256: &str) -> AiLocalServerBuild {
    AiLocalServerBuild {
        platform: platform.to_string(),
        url: format!("{SERVER_TAG_URL}/{archive}"),
        sha256: sha256.to_string(),
        size,
        executable: executable_of(platform).to_string(),
    }
}

/// Nom de l'exécutable : `.exe` sous Windows.
fn executable_of(platform: &str) -> &'static str {
    if platform.starts_with("windows") {
        "llama-server.exe"
    } else {
        "llama-server"
    }
}
