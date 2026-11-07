#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Opérations d'hôte et version du protocole : `retry_unlock` s'écrit comme `shutdown`, la mineure monte à 1.

use mira_protocol::{Frame, HostOp, PROTOCOL_VERSION};

/// `retry_unlock` est lu depuis le JSON du client puis réécrit à l'identique.
#[test]
fn retry_unlock_round_trips() {
    let text = r#"{"type":"host_op","id":6,"op":"retry_unlock"}"#;
    let frame: Frame = serde_json::from_str(text).unwrap();
    assert_eq!(
        frame,
        Frame::HostOp {
            id: 6,
            op: HostOp::RetryUnlock
        }
    );
    assert_eq!(serde_json::to_string(&frame).unwrap(), text);
}

/// La version du protocole annonce l'ajout compatible de la phase 4.
#[test]
fn protocol_version_is_one_dot_one() {
    assert_eq!((PROTOCOL_VERSION.major, PROTOCOL_VERSION.minor), (1, 1));
}

/// `set_presence` porte l'identifiant de l'échange et l'état du focus, à plat.
#[test]
fn set_presence_round_trips() {
    let text = r#"{"type":"set_presence","id":8,"focused":true}"#;
    let frame: Frame = serde_json::from_str(text).unwrap();
    assert_eq!(
        frame,
        Frame::SetPresence {
            id: 8,
            focused: true
        }
    );
    assert_eq!(serde_json::to_string(&frame).unwrap(), text);
}
