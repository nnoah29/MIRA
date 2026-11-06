#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Codec : préfixe de longueur, refus des tailles excessives avant le corps, JSON illisible.

use mira_protocol::{CodecError, Frame, MAX_FRAME_BYTES, body_len, decode, encode};

/// Trame d'exemple.
fn ping() -> Frame {
    Frame::Ping { nonce: 7 }
}

/// Le préfixe est la longueur du corps JSON sur quatre octets grand-boutistes.
#[test]
fn encode_prefixes_the_body_length_big_endian() {
    let bytes = encode(&ping(), MAX_FRAME_BYTES).unwrap();
    let announced = u32::from_be_bytes(bytes[..4].try_into().unwrap());
    assert_eq!(announced as usize, bytes.len() - 4);
    assert_eq!(&bytes[4..], br#"{"type":"ping","nonce":7}"#);
}

/// Le corps encodé se relit en la même trame.
#[test]
fn decode_returns_the_encoded_frame() {
    let bytes = encode(&ping(), MAX_FRAME_BYTES).unwrap();
    assert_eq!(decode(&bytes[4..]).unwrap(), ping());
}

/// Une taille annoncée trop grande est refusée sur l'en-tête seul, sans corps.
#[test]
fn an_announced_size_above_the_maximum_is_refused_from_the_header_alone() {
    let error = body_len([0xFF; 4], 1024).unwrap_err();
    assert_eq!(
        error,
        CodecError::TooLarge {
            announced: u32::MAX,
            max: 1024
        }
    );
    assert_eq!(body_len(1024u32.to_be_bytes(), 1024).unwrap(), 1024);
}

/// Écrire une trame plus grande que le maximum est refusé.
#[test]
fn encode_refuses_a_frame_above_the_maximum() {
    let error = encode(&ping(), 8).unwrap_err();
    assert!(matches!(error, CodecError::TooLarge { max: 8, .. }));
}

/// Une longueur nulle, un corps vide, du bruit ou un JSON coupé ne sont pas des trames.
#[test]
fn empty_garbage_and_truncated_bodies_are_errors() {
    assert_eq!(body_len([0; 4], 1024).unwrap_err(), CodecError::Empty);
    assert_eq!(decode(b"").unwrap_err(), CodecError::Empty);
    assert_eq!(decode(b"\xff\xfe nope").unwrap_err(), CodecError::Malformed);
    let cut = &br#"{"type":"ping","nonce":7}"#[..12];
    assert_eq!(decode(cut).unwrap_err(), CodecError::Malformed);
}
