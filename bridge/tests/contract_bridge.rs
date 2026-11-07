#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeu de contrat sur le chemin « pont FFI » : les scénarios sont ceux du démon, inclus tels quels.

mod bridge_driver;
#[path = "../../engine/crates/mira-daemon/tests/contract/mod.rs"]
mod contract;

use bridge_driver::BridgeDriver;

contract::suite_mail::mail_suite!(BridgeDriver);
contract::suite_chat::chat_suite!(BridgeDriver);
contract::suite_chat_errors::chat_errors_suite!(BridgeDriver);
contract::suite_chat_extras::chat_extras_suite!(BridgeDriver);
contract::suite_data::data_suite!(BridgeDriver);
