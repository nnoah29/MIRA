//! Écran de la boucle : dessin de l'état, suivi de la taille et copie des images pour les tests.

use ratatui::Terminal;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use tokio::sync::mpsc::UnboundedSender;

use super::error::TuiError;
use crate::state::AppState;
use crate::update::Msg;
use crate::view::render;

/// Terminal de la session et, éventuellement, destinataire d'une copie de chaque image.
pub(crate) struct Screen<'a, B: Backend> {
    pub(crate) terminal: &'a mut Terminal<B>,
    pub(crate) frames: Option<UnboundedSender<Buffer>>,
}

impl<B: Backend> Screen<'_, B> {
    /// Message de redimensionnement si la taille du terminal n'est plus celle de l'état.
    pub(crate) fn resized(&self, state: &AppState) -> Result<Option<Msg>, TuiError> {
        let size = self.terminal.size().map_err(terminal_error)?;
        let changed = (size.width, size.height) != (state.width, state.height);
        Ok(changed.then_some(Msg::Resize(size.width, size.height)))
    }

    /// Dessine l'état et envoie une copie de l'image si quelqu'un l'attend.
    pub(crate) fn draw(&mut self, state: &AppState) -> Result<(), TuiError> {
        let frame = self
            .terminal
            .draw(|frame| render(frame, state))
            .map_err(terminal_error)?;
        if let Some(frames) = &self.frames {
            let _ = frames.send(frame.buffer.clone());
        }
        Ok(())
    }
}

/// Erreur du terminal, quel que soit le type d'erreur de son pilote.
pub(crate) fn terminal_error(error: impl std::fmt::Display) -> TuiError {
    TuiError::Terminal(std::io::Error::other(error.to_string()))
}
