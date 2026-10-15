//! Touches de la vue Paramètres : communes, puis propres à la page affichée ; en formulaire, tout est du texte.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::keys::{is_typed, lookup};
use crate::state::AppState;
use crate::state::settings::SettingsTab;
use crate::update::settings::{AccountOp, FormOp, IdentityOp, MaintenanceOp, PageOp};
use crate::update::{Intent, PaletteIntent, SettingsIntent};

/// Intention d'ouvrir cette page.
const fn show(tab: SettingsTab) -> Intent {
    Intent::Settings(SettingsIntent::Show(tab))
}

const COMMON_KEYS: [(KeyCode, Intent); 15] = [
    (KeyCode::Char('j'), Intent::MoveDown),
    (KeyCode::Down, Intent::MoveDown),
    (KeyCode::Char('k'), Intent::MoveUp),
    (KeyCode::Up, Intent::MoveUp),
    (KeyCode::Tab, Intent::Settings(SettingsIntent::NextTab)),
    (KeyCode::BackTab, Intent::Settings(SettingsIntent::PrevTab)),
    (KeyCode::Char(']'), Intent::NextAccount),
    (KeyCode::Char('['), Intent::PrevAccount),
    (KeyCode::Char(':'), Intent::Palette(PaletteIntent::Open)),
    (KeyCode::Esc, Intent::Back),
    (KeyCode::Char('q'), Intent::Quit),
    (KeyCode::Char('1'), show(SettingsTab::Accounts)),
    (KeyCode::Char('2'), show(SettingsTab::Identities)),
    (KeyCode::Char('3'), show(SettingsTab::Maintenance)),
    (KeyCode::Char('4'), show(SettingsTab::Events)),
];

const ACCOUNT_KEYS: [(KeyCode, Intent); 6] = [
    (KeyCode::Char('g'), Intent::Top),
    (KeyCode::Char('G'), Intent::Bottom),
    (
        KeyCode::Char('E'),
        Intent::Settings(SettingsIntent::Account(AccountOp::Edit)),
    ),
    (
        KeyCode::Char('p'),
        Intent::Settings(SettingsIntent::Account(AccountOp::Pause)),
    ),
    (
        KeyCode::Char('r'),
        Intent::Settings(SettingsIntent::Account(AccountOp::Resume)),
    ),
    (
        KeyCode::Char('X'),
        Intent::Settings(SettingsIntent::Account(AccountOp::Delete)),
    ),
];

const IDENTITY_KEYS: [(KeyCode, Intent); 5] = [
    (KeyCode::Char('g'), Intent::Top),
    (KeyCode::Char('G'), Intent::Bottom),
    (
        KeyCode::Char('a'),
        Intent::Settings(SettingsIntent::Identity(IdentityOp::Add)),
    ),
    (
        KeyCode::Char('e'),
        Intent::Settings(SettingsIntent::Identity(IdentityOp::Edit)),
    ),
    (
        KeyCode::Char('d'),
        Intent::Settings(SettingsIntent::Identity(IdentityOp::SetDefault)),
    ),
];

const MAINTENANCE_KEYS: [(KeyCode, Intent); 6] = [
    (
        KeyCode::Char('s'),
        Intent::Settings(SettingsIntent::Maintenance(MaintenanceOp::Stats)),
    ),
    (
        KeyCode::Char('c'),
        Intent::Settings(SettingsIntent::Maintenance(MaintenanceOp::Check)),
    ),
    (
        KeyCode::Char('p'),
        Intent::Settings(SettingsIntent::Maintenance(MaintenanceOp::Purge)),
    ),
    (
        KeyCode::Char('i'),
        Intent::Settings(SettingsIntent::Maintenance(MaintenanceOp::Rebuild)),
    ),
    (
        KeyCode::Char('d'),
        Intent::Settings(SettingsIntent::Maintenance(MaintenanceOp::ExportDiagnostic)),
    ),
    (
        KeyCode::Char('x'),
        Intent::Settings(SettingsIntent::Maintenance(MaintenanceOp::ExportProfile)),
    ),
];

const EVENTS_KEYS: [(KeyCode, Intent); 7] = [
    (KeyCode::Char('g'), Intent::Top),
    (
        KeyCode::Char('G'),
        Intent::Settings(SettingsIntent::Page(PageOp::Last)),
    ),
    (
        KeyCode::Home,
        Intent::Settings(SettingsIntent::Page(PageOp::First)),
    ),
    (
        KeyCode::Char('n'),
        Intent::Settings(SettingsIntent::Page(PageOp::Next)),
    ),
    (
        KeyCode::PageDown,
        Intent::Settings(SettingsIntent::Page(PageOp::Next)),
    ),
    (
        KeyCode::Char('p'),
        Intent::Settings(SettingsIntent::Page(PageOp::Previous)),
    ),
    (
        KeyCode::PageUp,
        Intent::Settings(SettingsIntent::Page(PageOp::Previous)),
    ),
];

const FORM_KEYS: [(KeyCode, Intent); 4] = [
    (
        KeyCode::Backspace,
        Intent::Settings(SettingsIntent::Form(FormOp::Backspace)),
    ),
    (
        KeyCode::Tab,
        Intent::Settings(SettingsIntent::Form(FormOp::NextField)),
    ),
    (
        KeyCode::BackTab,
        Intent::Settings(SettingsIntent::Form(FormOp::PrevField)),
    ),
    (
        KeyCode::Enter,
        Intent::Settings(SettingsIntent::Form(FormOp::Submit)),
    ),
];

/// Intention d'une touche de la vue : les touches communes, puis celles de la page affichée.
pub fn intent(state: &AppState, key: KeyEvent) -> Option<Intent> {
    let page: &[(KeyCode, Intent)] = match state.settings.tab {
        SettingsTab::Accounts => &ACCOUNT_KEYS,
        SettingsTab::Identities => &IDENTITY_KEYS,
        SettingsTab::Maintenance => &MAINTENANCE_KEYS,
        SettingsTab::Events => &EVENTS_KEYS,
    };
    lookup(&COMMON_KEYS, key).or_else(|| lookup(page, key))
}

/// Intention d'une touche dans un formulaire : tout caractère est du texte, Échap ferme.
pub fn form_intent(key: KeyEvent) -> Option<Intent> {
    if let KeyCode::Char(c) = key.code {
        let text = Intent::Settings(SettingsIntent::Form(FormOp::Char(c)));
        return is_typed(key).then_some(text);
    }
    if key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT) {
        return Some(Intent::Settings(SettingsIntent::Form(FormOp::PrevField)));
    }
    if key.code == KeyCode::Esc {
        return Some(Intent::Cancel);
    }
    lookup(&FORM_KEYS, key)
}
