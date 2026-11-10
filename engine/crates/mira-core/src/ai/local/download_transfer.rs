//! Transfert réseau d'un fichier épinglé : requête (avec `Range` à la reprise), écriture par blocs, annulation.

use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Duration;

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};
use reqwest::redirect::Policy;
use reqwest::{Client, Response, StatusCode};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use super::download::{CancelFlag, DownloadJob, io_error};
use super::download_errors::{cancelled_error, connection_lost};
use super::download_verify::mismatch;
use crate::auth::loopback::is_loopback_host;

/// Taille du fichier partiel s'il existe et n'excède pas la taille attendue ; sinon il est supprimé.
pub(super) async fn part_size(part: &Path, expected: u64) -> u64 {
    let size = tokio::fs::metadata(part)
        .await
        .map(|m| m.len())
        .unwrap_or(0);
    if size > expected {
        let _ = tokio::fs::remove_file(part).await;
        return 0;
    }
    size
}

/// Client qui suit au plus cinq redirections et n'ajoute aucun en-tête d'authentification.
fn client_for(url: &str) -> Result<Client, MiraError> {
    let parsed = reqwest::Url::parse(url).map_err(|_| io_error())?;
    let mut builder = Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .read_timeout(Duration::from_secs(60))
        .redirect(Policy::limited(5));
    if parsed.host_str().is_some_and(is_loopback_host) {
        builder = builder.no_proxy();
    }
    builder.build().map_err(|_| connection_lost())
}

/// Envoie la requête, avec `Range` si une partie existe, et vérifie le statut.
async fn open(job: &DownloadJob<'_>, existing: u64) -> Result<Response, MiraError> {
    let mut request = client_for(job.url)?.get(job.url);
    if existing > 0 {
        request = request.header("Range", format!("bytes={existing}-"));
    }
    let response = request.send().await.map_err(|_| connection_lost())?;
    match response.status() {
        StatusCode::OK | StatusCode::PARTIAL_CONTENT => Ok(response),
        _ => Err(MiraError::new(
            ErrorCategory::Server,
            server::REJECTED,
            "l'hébergeur a refusé le téléchargement",
        )),
    }
}

/// Ouvre le fichier partiel en ajout si le serveur reprend, sinon le recrée vide.
async fn open_part(part: &Path, resumed: bool) -> Result<tokio::fs::File, MiraError> {
    let mut options = OpenOptions::new();
    options.create(true);
    if resumed {
        options.append(true);
    } else {
        options.write(true).truncate(true);
    }
    options.open(part).await.map_err(|_| io_error())
}

/// Télécharge le reste du fichier dans `part` ; un transfert coupé ou annulé garde le fichier partiel.
pub(super) async fn fetch_into(
    job: &DownloadJob<'_>,
    part: &Path,
    existing: u64,
    progress: &(dyn Fn(u64, u64) + Send + Sync),
    cancel: &CancelFlag,
) -> Result<(), MiraError> {
    let mut response = open(job, existing).await?;
    let resumed = response.status() == StatusCode::PARTIAL_CONTENT;
    let mut file = open_part(part, resumed).await?;
    let mut done = if resumed { existing } else { 0 };
    while let Some(chunk) = response.chunk().await.map_err(|_| connection_lost())? {
        if cancel.load(Ordering::SeqCst) {
            return Err(cancelled_error());
        }
        file.write_all(&chunk).await.map_err(|_| io_error())?;
        done += chunk.len() as u64;
        progress(done, job.size);
        if done > job.size {
            let _ = tokio::fs::remove_file(part).await;
            return Err(mismatch());
        }
    }
    file.flush().await.map_err(|_| io_error())?;
    Ok(())
}
