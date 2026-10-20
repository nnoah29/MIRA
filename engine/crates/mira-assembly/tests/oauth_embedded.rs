#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Identifiants OAuth intégrés : déchiffrement AES-256-GCM, refus d'un contenu altéré, priorité du fichier local.

use mira_assembly::mira_vocabulary::ProviderId;
use mira_assembly::oauth_embedded::open_embedded;
use mira_assembly::oauth_file_parse::parse_oauth_config;
use mira_assembly::oauth_overlay::overlay;
use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};

const KEY: [u8; 32] = [7; 32];
const NONCE: [u8; 12] = [3; 12];

/// Scelle le texte comme le fait build.rs
fn seal(text: &str) -> Vec<u8> {
    let key = LessSafeKey::new(UnboundKey::new(&AES_256_GCM, &KEY).unwrap());
    let mut buffer = text.as_bytes().to_vec();
    key.seal_in_place_append_tag(
        Nonce::assume_unique_for_key(NONCE),
        Aad::empty(),
        &mut buffer,
    )
    .unwrap();
    buffer
}

/// Une configuration scellée se rouvre avec sa clé, sans que l'identifiant apparaisse en clair dans le texte chiffré
#[test]
fn sealed_config_round_trips() {
    let sealed = seal("[oauth.gmail]\nclient_id = \"id-123.apps\"\nclient_secret = \"s3cr3t\"\n");
    assert!(!sealed.windows(6).any(|window| window == b"id-123"));
    let config = open_embedded(&KEY, &NONCE, &sealed).unwrap();
    let gmail = ProviderId("gmail".into());
    assert_eq!(config.client_ids[&gmail], "id-123.apps");
    assert_eq!(config.client_secrets[&gmail].expose(), "s3cr3t");
}

/// Une clé fausse ou un octet altéré sont refusés sans citer le contenu
#[test]
fn wrong_key_or_tampering_is_rejected() {
    let sealed = seal("[oauth.gmail]\nclient_id = \"id-123\"\n");
    let error = open_embedded(&[8; 32], &NONCE, &sealed).unwrap_err();
    assert!(!format!("{error:?}").contains("id-123"));
    let mut tampered = sealed.clone();
    tampered[0] ^= 1;
    assert!(open_embedded(&KEY, &NONCE, &tampered).is_err());
    assert!(open_embedded(&KEY[..16], &NONCE, &sealed).is_err());
}

/// Le fichier local remplace le provider qu'il déclare (secret intégré compris) et laisse les autres intacts
#[test]
fn local_file_overrides_embedded_per_provider() {
    let embedded = parse_oauth_config(
        "[oauth.gmail]\nclient_id = \"g-built\"\nclient_secret = \"g-sec\"\n[oauth.outlook]\nclient_id = \"o-built\"\n",
    )
    .unwrap();
    let local = parse_oauth_config("[oauth.gmail]\nclient_id = \"g-local\"\n").unwrap();
    let merged = overlay(embedded, local);
    let gmail = ProviderId("gmail".into());
    assert_eq!(merged.client_ids[&gmail], "g-local");
    assert!(!merged.client_secrets.contains_key(&gmail));
    assert_eq!(merged.client_ids[&ProviderId("outlook".into())], "o-built");
}
