#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeu de contrat sur le chemin « hôte direct » : le moteur est ouvert dans le processus, sans transport.

mod contract;
mod drivers;
mod support;

use drivers::direct::DirectDriver;

contract::suite_mail::mail_suite!(DirectDriver);
contract::suite_chat::chat_suite!(DirectDriver);
contract::suite_chat_errors::chat_errors_suite!(DirectDriver);
contract::suite_chat_extras::chat_extras_suite!(DirectDriver);
contract::suite_data::data_suite!(DirectDriver);
