//! Intentions de rédaction, indépendantes des touches qui les produisent.

/// Touche d'édition du texte du champ qui a le focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKey {
    Char(char),
    Newline,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

/// Ce que l'utilisateur veut faire d'un brouillon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposeIntent {
    Reply,
    ReplyAll,
    Forward,
    New,
    Edit(EditKey),
    NextField,
    PrevField,
    Save,
    Close,
    SaveAndClose,
    Discard,
}
