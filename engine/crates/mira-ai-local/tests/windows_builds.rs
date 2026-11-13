#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Archives Windows du serveur : épinglées par tag, taille et somme, exécutable en `.exe`.

use mira_ai_local::LocalAiProvider;
use mira_vocabulary::AiProvider;

/// Les deux archives Windows (CPU) sont figées ; l'exécutable porte l'extension `.exe`
#[test]
fn windows_server_builds_are_pinned() {
    let runtime = LocalAiProvider.descriptor().local_runtime.unwrap();
    let base = "https://github.com/ggml-org/llama.cpp/releases/download/b12224/";
    let cases = [
        (
            "windows-x86_64",
            "llama-b12224-bin-win-cpu-x64.zip",
            24_887_408,
            "d7a7a3e014a02dbfce46977f9da6d9b58196b9c83b80bcf6ef2337134103c3d6",
        ),
        (
            "windows-aarch64",
            "llama-b12224-bin-win-cpu-arm64.zip",
            13_237_558,
            "7d3cb8f7076337c51b5182ad45cfc2b12f0f2ed30efa66196f2f6f0b68a6c23d",
        ),
    ];
    for (platform, file, size, sha) in cases {
        let build = runtime
            .server
            .iter()
            .find(|b| b.platform == platform)
            .unwrap();
        assert_eq!(build.url, format!("{base}{file}"));
        assert_eq!((build.size, build.sha256.as_str()), (size, sha));
        assert_eq!(build.executable, "llama-server.exe");
    }
}

/// Les builds Linux gardent un exécutable sans extension
#[test]
fn linux_builds_keep_a_plain_executable_name() {
    let runtime = LocalAiProvider.descriptor().local_runtime.unwrap();
    for build in runtime
        .server
        .iter()
        .filter(|b| b.platform.starts_with("linux"))
    {
        assert_eq!(build.executable, "llama-server");
    }
}
