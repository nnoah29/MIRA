#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeu de contrat sur le chemin « socket » : le moteur est derrière le démon, atteint par le protocole.

mod contract;
mod drivers;
mod support;

use drivers::socket::SocketDriver;

contract::suite_mail::mail_suite!(SocketDriver);
contract::suite_chat::chat_suite!(SocketDriver);
contract::suite_chat_errors::chat_errors_suite!(SocketDriver);
contract::suite_chat_extras::chat_extras_suite!(SocketDriver);
contract::suite_data::data_suite!(SocketDriver);
