#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeu de contrat sur le chemin « socket » : le moteur est derrière le démon, atteint par le protocole.

mod contract;
mod drivers;
mod support;

use drivers::socket::SocketDriver;

contract::suite::mail_suite!(SocketDriver);
