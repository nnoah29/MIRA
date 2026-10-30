#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Confidentialité de bout en bout : refus par type, masquage, restitution et journal sans texte, moteur assemblé.

mod ai_support;

use ai_support::{
    Fixture, KEY, answer_text, ask_ai, chat_with, collect, connection_ready, events_text,
    journal_rows, start,
};
use mira_assembly::api::{AiRequest, AiResponse};
use mira_assembly::mira_vocabulary::{
    AiConnection, AiProviderId, ChatDelta, ContentKind, ContextPart, MaskingMode, MaskingSettings,
    SensitiveCategory,
};
use mira_testkit::ai::openai_stream;

const IBAN: &str = "FR7630006000011234567890189";
const SUBJECT: &str = "SUJET-CONFIDENTIEL-31";
const QUESTION: &str = "QUESTION-PRIVEE-47";
const REPLY: &str = "REPONSE-PRIVEE-53";

/// Partie de contexte d'un mail dont le corps contient l'IBAN
fn mail_part() -> Vec<ContextPart> {
    vec![ContextPart {
        kind: ContentKind::MailMessage,
        label: SUBJECT.to_string(),
        text: format!("Mon IBAN est {IBAN}, merci."),
    }]
}

/// Connexion vers la boucle locale dont le masquage est actif sur l'IBAN
async fn masked_connection(f: &Fixture) -> AiConnection {
    let connection = connection_ready(f).await;
    let masking = MaskingSettings {
        mode: MaskingMode::Mask,
        categories: vec![SensitiveCategory::Iban],
    };
    let set = AiRequest::SetAiMasking {
        connection: connection.id.clone(),
        masking,
    };
    let AiResponse::Connection(masked) = ask_ai(f, set).await else {
        panic!("connexion attendue");
    };
    masked
}

/// Connexion distante (jamais jointe) dont le consentement est donné sans aucun type autorisé
async fn remote_connection(f: &Fixture) -> AiConnection {
    let add = AiRequest::AddAiConnection {
        provider: AiProviderId("generic".to_string()),
        label: "Distante".to_string(),
        base_url: Some("https://ia.example.org/v1".to_string()),
    };
    let AiResponse::Connection(remote) = ask_ai(f, add).await else {
        panic!("connexion attendue");
    };
    let consent = AiRequest::AcknowledgeAiConsent {
        connection: remote.id.clone(),
        kinds: Vec::new(),
    };
    ask_ai(f, consent).await;
    remote
}

/// Une connexion distante refuse le type tant qu'il n'est pas autorisé, puis le laisse passer ; le même parcours part vers la boucle locale
#[tokio::test]
async fn refused_kind_then_allowed_then_sent() {
    let f = start().await;
    let remote = remote_connection(&f).await;
    assert_eq!(remote.masking.mode, MaskingMode::Mask);
    let refused = chat_with(&f, &remote.id, QUESTION, mail_part()).await;
    let code = "incompatible_state.ai_content_not_allowed";
    assert_eq!(refused.unwrap_err().code, code);
    let kinds = vec![ContentKind::MailMessage];
    let allow = AiRequest::SetAiAllowedContent {
        connection: remote.id.clone(),
        kinds,
    };
    ask_ai(&f, allow).await;
    let next = chat_with(&f, &remote.id, QUESTION, mail_part()).await;
    assert_eq!(next.unwrap_err().code, "incompatible_state.ai_key_missing");
    assert!(f.server.requests().is_empty());
    let local = masked_connection(&f).await;
    f.server.queue(openai_stream(&["ok"], None));
    let mut stream = chat_with(&f, &local.id, QUESTION, mail_part()).await;
    collect(stream.as_mut().unwrap()).await;
    assert_eq!(f.server.requests().len(), 1);
}

/// Le serveur ne voit pas l'IBAN, la réponse le retrouve, le journal compte la valeur masquée
#[tokio::test]
async fn full_flow_masks_logs_and_restores() {
    let f = start().await;
    let connection = masked_connection(&f).await;
    let chunks = ["Votre IBAN [IBAN", "_1] est noté."];
    f.server.queue(openai_stream(&chunks, Some((9, 4))));
    let mut stream = chat_with(&f, &connection.id, QUESTION, mail_part()).await;
    let deltas = collect(stream.as_mut().unwrap()).await;
    let body = &f.server.requests()[0].body;
    assert!(!body.contains(IBAN) && body.contains("[IBAN_1]"));
    assert_eq!(deltas[0], ChatDelta::Masked { count: 1 });
    assert_eq!(answer_text(&deltas), format!("Votre IBAN {IBAN} est noté."));
    let rows = journal_rows(&f).await;
    assert_eq!((rows.len(), rows[0].masked, rows[0].parts), (1, 1, 1));
    assert_eq!(rows[0].kinds, [ContentKind::MailMessage]);
    assert!(rows[0].local && rows[0].outcome == "end_turn");
}

/// Après un parcours complet, ni le texte envoyé, ni la réponse, ni l'IBAN, ni la clé ne sont dans le journal ni dans les événements
#[tokio::test]
async fn journal_has_no_prompt_text_after_a_full_flow() {
    let f = start().await;
    let connection = masked_connection(&f).await;
    f.server
        .queue(openai_stream(&[REPLY, " [IBAN_1]"], Some((9, 4))));
    let mut stream = chat_with(&f, &connection.id, QUESTION, mail_part()).await;
    collect(stream.as_mut().unwrap()).await;
    let rows = format!("{:?}", journal_rows(&f).await);
    let events = events_text(&f).await;
    for hidden in [IBAN, SUBJECT, QUESTION, REPLY, KEY, "Mon IBAN est"] {
        assert!(!rows.contains(hidden), "{hidden} dans {rows}");
        assert!(!events.contains(hidden), "{hidden} dans {events}");
    }
}
