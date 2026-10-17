//! Flux IMAP : transport, tampon de lecture, étiquettes et délai d'attente.

use std::fmt;
use std::time::Duration;

use mira_vocabulary::MiraError;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};

use crate::imap::errors::network::{from_io, timeout};

/// Transport chiffré ou en clair, rendu interchangeable.
pub(crate) trait Transport: AsyncRead + AsyncWrite + Unpin + Send {}

impl<T: AsyncRead + AsyncWrite + Unpin + Send> Transport for T {}

/// Transport emballé, indépendant de TCP ou TLS.
pub(crate) type BoxedTransport = Box<dyn Transport>;

/// Connexion IMAP : lit des lignes logiques et écrit des commandes étiquetées.
pub(crate) struct Wire {
    pub(super) reader: BufReader<ReadHalf<BoxedTransport>>,
    pub(super) writer: WriteHalf<BoxedTransport>,
    pub(super) pending: Vec<u8>,
    pub(super) next_tag: u32,
    pub(crate) wait: Duration,
    pub(super) literal_plus: bool,
    pub(super) response_bytes: usize,
    pub(super) broken: bool,
}

impl fmt::Debug for Wire {
    /// Affiche l'état du flux sans jamais montrer d'octets échangés.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wire")
            .field("next_tag", &self.next_tag)
            .field("wait", &self.wait)
            .finish_non_exhaustive()
    }
}

impl Wire {
    /// Prépare un flux sur le transport donné, avec le délai d'attente de chaque échange.
    pub(crate) fn new(transport: BoxedTransport, wait: Duration) -> Self {
        let (read, write) = tokio::io::split(transport);
        Self {
            reader: BufReader::new(read),
            writer: write,
            pending: Vec::new(),
            next_tag: 0,
            wait,
            literal_plus: false,
            response_bytes: 0,
            broken: false,
        }
    }

    /// Reprend le transport brut, par exemple pour l'envelopper dans TLS après STARTTLS.
    pub(crate) fn into_transport(self) -> BoxedTransport {
        self.reader.into_inner().unsplit(self.writer)
    }

    /// Mémorise si les littéraux peuvent être envoyés sans attendre le serveur (LITERAL+).
    pub(crate) fn set_literal_plus(&mut self, enabled: bool) {
        self.literal_plus = enabled;
    }

    /// Fabrique l'étiquette de la prochaine commande.
    pub(crate) fn fresh_tag(&mut self) -> String {
        self.next_tag += 1;
        self.response_bytes = 0;
        format!("A{:04}", self.next_tag)
    }

    /// Écrit des octets et vide le tampon, dans le délai d'attente.
    pub(crate) async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), MiraError> {
        let sending = async {
            self.writer.write_all(bytes).await?;
            self.writer.flush().await
        };
        match tokio::time::timeout(self.wait, sending).await {
            Err(_) => Err(timeout()),
            Ok(result) => result.map_err(|error| from_io(&error)),
        }
    }
}
