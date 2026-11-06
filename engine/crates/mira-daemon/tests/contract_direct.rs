#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeu de contrat sur le chemin « hôte direct » : le moteur est ouvert dans le processus, sans transport.

mod contract;
mod drivers;
mod support;

use drivers::direct::DirectDriver;

contract::suite::mail_suite!(DirectDriver);
