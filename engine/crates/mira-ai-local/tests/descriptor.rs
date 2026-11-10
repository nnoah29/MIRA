#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La fiche du fournisseur « local » décrit un serveur et des modèles épinglés, sans secret.

use mira_ai_local::LocalAiProvider;
use mira_vocabulary::{AiLocalModel, AiProvider, AiProviderId, AuthScheme, LocalTier, WireFormat};

fn model(id: &str) -> AiLocalModel {
    let runtime = LocalAiProvider.descriptor().local_runtime.unwrap();
    runtime.models.into_iter().find(|m| m.id == id).unwrap()
}

/// La fiche annonce un serveur compatible OpenAI Chat sur la boucle locale, sans clé ni connexion
#[test]
fn local_descriptor_has_expected_values() {
    let d = LocalAiProvider.descriptor();
    assert_eq!(d.id, AiProviderId("local".to_string()));
    assert_eq!(d.display_name, "Modèle local");
    assert_eq!(d.base_url, "http://127.0.0.1");
    assert_eq!(d.auth, AuthScheme::Bearer);
    assert_eq!(d.format, WireFormat::OpenAiChat);
    assert_eq!(d.models_path, "/v1/models");
    assert!(!d.base_url_editable);
    assert!(!d.supports_tools);
    assert!(d.key_login.is_none());
    assert!(d.session_login.is_none());
}

/// Le serveur est épinglé par tag, URL, taille et somme pour chaque plateforme prise en charge
#[test]
fn pinned_server_builds_match_spec() {
    let runtime = LocalAiProvider.descriptor().local_runtime.unwrap();
    let base = "https://github.com/ggml-org/llama.cpp/releases/download/b12224/";
    let x64 = runtime
        .server
        .iter()
        .find(|b| b.platform == "linux-x86_64")
        .unwrap();
    assert_eq!(x64.url, format!("{base}llama-b12224-bin-ubuntu-x64.tar.gz"));
    assert_eq!(x64.size, 23_109_432);
    assert_eq!(
        x64.sha256,
        "e3f9e11e61b8bcecccc4b7522d131cc6d3ec18fc3f31f6caa58ddda63801bbd5"
    );
    let arm = runtime
        .server
        .iter()
        .find(|b| b.platform == "linux-aarch64")
        .unwrap();
    assert_eq!(
        arm.url,
        format!("{base}llama-b12224-bin-ubuntu-arm64.tar.gz")
    );
    assert_eq!(arm.size, 18_759_680);
    assert_eq!(
        arm.sha256,
        "9e66367e598fb0c7a196e3b6586b9c0b455fec0f5364b962a05ffd53aebc14be"
    );
    assert_eq!(runtime.server.len(), 2);
}

/// Le palier Mini est Granite 4.0 H 1B, sans outils, contexte 4096
#[test]
fn mini_model_matches_spec() {
    let m = model("granite-4.0-h-1b");
    assert_eq!(m.tier, LocalTier::Mini);
    assert_eq!(
        m.url,
        "https://huggingface.co/ibm-granite/granite-4.0-h-1b-GGUF/resolve/main/granite-4.0-h-1b-Q4_K_M.gguf"
    );
    assert_eq!(m.file_name, "granite-4.0-h-1b-Q4_K_M.gguf");
    assert_eq!(m.size, 901_162_208);
    assert_eq!(
        m.sha256,
        "da3d737121a96f3c9a316685212376257a7f167b74380855666dd488d6af3bcb"
    );
    assert_eq!((m.ctx_size, m.ram_recommended_mib), (4096, 1300));
    assert!(!m.supports_tools && !m.reasoning);
    assert_eq!(m.license, "Apache-2.0");
}

/// Le palier Léger est Ministral 3 3B, sans outils, contexte 8192
#[test]
fn light_model_matches_spec() {
    let m = model("ministral-3-3b");
    assert_eq!(m.tier, LocalTier::Light);
    assert_eq!(
        m.url,
        "https://huggingface.co/mistralai/Ministral-3-3B-Instruct-2512-GGUF/resolve/main/Ministral-3-3B-Instruct-2512-Q4_K_M.gguf"
    );
    assert_eq!(m.size, 2_147_023_008);
    assert_eq!(
        m.sha256,
        "9ed150d4367e68df0ac8e1540f6ddc65b42d0ee26378329d1ecbca60f93fc5f8"
    );
    assert_eq!((m.ctx_size, m.ram_recommended_mib), (8192, 3500));
    assert!(!m.supports_tools && !m.reasoning);
}

/// Le palier Avancé est Granite 4.2 8B, avec outils, contexte 8192
#[test]
fn advanced_model_matches_spec() {
    let m = model("granite-4.2-8b");
    assert_eq!(m.tier, LocalTier::Advanced);
    assert_eq!(
        m.url,
        "https://huggingface.co/ibm-granite/granite-4.2-8b-GGUF/resolve/main/granite-4.2-8b-Q4_K_M.gguf"
    );
    assert_eq!(m.size, 5_347_917_952);
    assert_eq!(
        m.sha256,
        "16a9369d0805f80b7377d25d87f937a90c05dc04ad79173a52001e42c9aab311"
    );
    assert_eq!((m.ctx_size, m.ram_recommended_mib), (8192, 7000));
    assert!(m.supports_tools && !m.reasoning);
}

/// Toute somme est en hexadécimal minuscule de 64 caractères et toute adresse est en HTTPS
#[test]
fn every_hash_is_64_lowercase_hex_and_every_url_is_https() {
    let runtime = LocalAiProvider.descriptor().local_runtime.unwrap();
    let hashes = runtime
        .server
        .iter()
        .map(|b| (&b.sha256, &b.url))
        .chain(runtime.models.iter().map(|m| (&m.sha256, &m.url)));
    for (hash, url) in hashes {
        assert_eq!(hash.len(), 64, "{hash}");
        assert!(
            hash.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')),
            "{hash}"
        );
        assert!(url.starts_with("https://"), "{url}");
    }
}

/// Le gabarit n'emploie que des jetons connus et ne demande ni outils intégrés ni interface web
#[test]
fn placeholders_in_args_are_known_and_no_unsafe_flag_is_present() {
    let runtime = LocalAiProvider.descriptor().local_runtime.unwrap();
    let known = [
        "model",
        "host",
        "port",
        "api_key_file",
        "ctx",
        "threads",
        "alias",
    ];
    for arg in &runtime.server_args {
        if let Some(name) = arg.strip_prefix('{').and_then(|a| a.strip_suffix('}')) {
            assert!(known.contains(&name), "jeton inconnu {name}");
        }
        assert!(!["--tools", "--agent", "--webui", "--api-key"].contains(&arg.as_str()));
    }
    assert!(runtime.server_args.contains(&"--no-webui".to_string()));
    assert_eq!(runtime.health_path, "/health");
    assert_eq!(
        (runtime.idle_stop_seconds, runtime.start_timeout_seconds),
        (300, 120)
    );
    assert_eq!(runtime.reasoning_off_args, ["--reasoning", "off"]);
}
