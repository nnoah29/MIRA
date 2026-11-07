//! Droits d'une session : complets, ou limités à la sortie d'une mise à jour quand le moteur est périmé.

use mira_protocol::{Frame, HostOp};

/// Ce qu'un client attaché a le droit de demander.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Full,
    UpdateOnly,
}

impl Access {
    /// Dit si cette trame du client est permise avec ces droits.
    pub fn permits(self, frame: &Frame) -> bool {
        match self {
            Self::Full => true,
            Self::UpdateOnly => matches!(
                frame,
                Frame::Ping { .. }
                    | Frame::Pong { .. }
                    | Frame::Unknown
                    | Frame::HostOp {
                        op: HostOp::Shutdown,
                        ..
                    }
            ),
        }
    }
}
