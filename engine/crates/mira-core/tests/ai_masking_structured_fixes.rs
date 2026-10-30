#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Clé PEM non terminée et jetons à points après un mot-clé.

mod support_masking;

use mira_vocabulary::SensitiveCategory::Secret;
use support_masking::values;

/// Un bloc de clé privée sans ligne de fin est masqué jusqu'à la fin du texte
#[test]
fn unterminated_private_key_block_is_masked_to_the_end() {
    let text = "avant\n-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA\nabcdef";
    assert_eq!(
        values(text, &[Secret]),
        ["-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA\nabcdef"]
    );
    let closed = "-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----\nmerci";
    assert_eq!(
        values(closed, &[Secret]),
        ["-----BEGIN PRIVATE KEY-----\nAAAA\n-----END PRIVATE KEY-----"]
    );
}

/// Un jeton à points après un mot-clé est trouvé en entier
#[test]
fn dotted_token_after_a_keyword_is_found() {
    let dotted = "token=ya29.a0AfH6SMBx9Kq2LmN4pQ7rT1vWz";
    assert_eq!(
        values(dotted, &[Secret]),
        ["ya29.a0AfH6SMBx9Kq2LmN4pQ7rT1vWz"]
    );
    let header = "Authorization: aB3dE5fG7hJ9kL1mN3pQ.sT5uV7wX9yZ1aC3eF5gH7";
    assert_eq!(
        values(header, &[Secret]),
        ["aB3dE5fG7hJ9kL1mN3pQ.sT5uV7wX9yZ1aC3eF5gH7"]
    );
}

/// Le point de fin de phrase reste dehors et une phrase après un mot-clé n'est pas masquée
#[test]
fn sentence_period_stays_outside_and_prose_is_not_masked() {
    let ended = "Mon token: aB3dE5fG7hJ9kL1mN3pQsT5uV7wX9y.";
    assert_eq!(values(ended, &[Secret]), ["aB3dE5fG7hJ9kL1mN3pQsT5uV7wX9y"]);
    let prose = "secret: voir.le.document.joint.ci.dessous.";
    assert!(values(prose, &[Secret]).is_empty());
}
