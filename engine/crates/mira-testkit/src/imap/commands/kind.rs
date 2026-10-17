//! Classement des commandes selon l'état de session qu'elles exigent.

/// Commandes valables dans tous les états.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnyCommand {
    Capability,
    Noop,
    Logout,
    Id,
    Login,
    Authenticate,
}

/// Commandes qui exigent une session authentifiée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthCommand {
    Enable,
    List,
    Status,
    Create,
    Delete,
    Rename,
    Select,
    Examine,
    Append,
    Idle,
}

/// Commandes qui exigent une boîte sélectionnée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SelectedCommand {
    Fetch,
    Search,
    Store,
    Copy,
    Move,
    Expunge,
    Close,
    Check,
}

/// Commande reconnue, rangée selon l'état exigé.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommandKind {
    Any(AnyCommand),
    Auth(AuthCommand),
    Selected(SelectedCommand),
    Unknown,
}

impl CommandKind {
    /// Classe une commande ; seules certaines commandes acceptent le préfixe UID
    pub(crate) fn parse(name: &str, uid: bool) -> Self {
        let kind = parse_any(name)
            .map(Self::Any)
            .or_else(|| parse_auth(name).map(Self::Auth))
            .or_else(|| parse_selected(name).map(Self::Selected))
            .unwrap_or(Self::Unknown);
        match (uid, kind) {
            (true, Self::Selected(SelectedCommand::Close | SelectedCommand::Check)) => {
                Self::Unknown
            }
            (true, Self::Selected(_)) | (false, _) => kind,
            (true, _) => Self::Unknown,
        }
    }
}

/// Commandes valables dans tous les états
fn parse_any(name: &str) -> Option<AnyCommand> {
    match name {
        "CAPABILITY" => Some(AnyCommand::Capability),
        "NOOP" => Some(AnyCommand::Noop),
        "LOGOUT" => Some(AnyCommand::Logout),
        "ID" => Some(AnyCommand::Id),
        "LOGIN" => Some(AnyCommand::Login),
        "AUTHENTICATE" => Some(AnyCommand::Authenticate),
        _ => None,
    }
}

/// Commandes de l'état authentifié
fn parse_auth(name: &str) -> Option<AuthCommand> {
    match name {
        "ENABLE" => Some(AuthCommand::Enable),
        "LIST" => Some(AuthCommand::List),
        "STATUS" => Some(AuthCommand::Status),
        "CREATE" => Some(AuthCommand::Create),
        "DELETE" => Some(AuthCommand::Delete),
        "RENAME" => Some(AuthCommand::Rename),
        "SELECT" => Some(AuthCommand::Select),
        "EXAMINE" => Some(AuthCommand::Examine),
        "APPEND" => Some(AuthCommand::Append),
        "IDLE" => Some(AuthCommand::Idle),
        _ => None,
    }
}

/// Commandes de l'état sélectionné
fn parse_selected(name: &str) -> Option<SelectedCommand> {
    match name {
        "FETCH" => Some(SelectedCommand::Fetch),
        "SEARCH" => Some(SelectedCommand::Search),
        "STORE" => Some(SelectedCommand::Store),
        "COPY" => Some(SelectedCommand::Copy),
        "MOVE" => Some(SelectedCommand::Move),
        "EXPUNGE" => Some(SelectedCommand::Expunge),
        "CLOSE" => Some(SelectedCommand::Close),
        "CHECK" => Some(SelectedCommand::Check),
        _ => None,
    }
}
