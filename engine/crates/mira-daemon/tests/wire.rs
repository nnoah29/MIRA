#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecture et écriture de trames sur un flux d'octets : fins propres, tailles, troncatures.

use std::time::Duration;

use mira_daemon::wire::{WireError, read_frame, write_frame};
use mira_protocol::{Frame, MAX_FRAME_BYTES};
use tokio::io::{AsyncWriteExt, duplex};

/// Une trame écrite se relit à l'identique.
#[tokio::test]
async fn a_frame_written_is_read_back() {
    let (mut a, mut b) = duplex(4096);
    write_frame(&mut a, &Frame::Ping { nonce: 9 }, MAX_FRAME_BYTES)
        .await
        .unwrap();
    let read = read_frame(&mut b, MAX_FRAME_BYTES).await.unwrap();
    assert_eq!(read, Some(Frame::Ping { nonce: 9 }));
}

/// Une fin de flux entre deux trames se lit comme `None`.
#[tokio::test]
async fn a_clean_end_before_a_frame_reads_as_none() {
    let (a, mut b) = duplex(4096);
    drop(a);
    assert!(read_frame(&mut b, MAX_FRAME_BYTES).await.unwrap().is_none());
}

/// Quatre octets annonçant 4 Gio sont refusés sans attendre de corps.
#[tokio::test]
async fn an_announced_4_gib_is_refused_without_reading_the_body() {
    let (mut a, mut b) = duplex(4096);
    a.write_all(&[0xFF; 4]).await.unwrap();
    let read = tokio::time::timeout(Duration::from_secs(1), read_frame(&mut b, MAX_FRAME_BYTES));
    let error = read
        .await
        .expect("la lecture ne doit pas attendre le corps");
    assert!(matches!(error, Err(WireError::TooLarge { max, .. }) if max == MAX_FRAME_BYTES));
}

/// Une taille au-delà du maximum du serveur est refusée, même modeste.
#[tokio::test]
async fn a_size_above_the_server_maximum_is_refused() {
    let (mut a, mut b) = duplex(4096);
    write_frame(&mut a, &Frame::Ping { nonce: 9 }, MAX_FRAME_BYTES)
        .await
        .unwrap();
    let error = read_frame(&mut b, 8).await.unwrap_err();
    assert!(matches!(error, WireError::TooLarge { max: 8, .. }));
}

/// Un corps coupé par la fin du flux est une troncature.
#[tokio::test]
async fn a_body_cut_short_is_truncated() {
    let (mut a, mut b) = duplex(4096);
    a.write_all(&[0, 0, 0, 20]).await.unwrap();
    a.write_all(b"{\"type\":").await.unwrap();
    drop(a);
    let error = read_frame(&mut b, MAX_FRAME_BYTES).await.unwrap_err();
    assert!(matches!(error, WireError::Truncated));
}

/// Un corps vide ou du bruit ne sont pas des trames ; écrire au-delà du maximum est refusé.
#[tokio::test]
async fn empty_or_garbage_bodies_are_malformed_and_oversize_writes_are_refused() {
    let (mut a, mut b) = duplex(4096);
    a.write_all(&[0, 0, 0, 0]).await.unwrap();
    let empty = read_frame(&mut b, MAX_FRAME_BYTES).await.unwrap_err();
    assert!(matches!(empty, WireError::Malformed));
    a.write_all(&[0, 0, 0, 4]).await.unwrap();
    a.write_all(b"nope").await.unwrap();
    let garbage = read_frame(&mut b, MAX_FRAME_BYTES).await.unwrap_err();
    assert!(matches!(garbage, WireError::Malformed));
    let refused = write_frame(&mut a, &Frame::Ping { nonce: 9 }, 8)
        .await
        .unwrap_err();
    assert!(matches!(refused, WireError::TooLarge { .. }));
}
