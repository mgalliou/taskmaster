use std::process::Child;

use crate::config::ProgramConfig;

pub mod start;
pub mod status;
pub mod stop;
pub mod restart;
pub mod reload;
pub mod exit;

pub type ProcessInfo = Vec<(ProgramConfig, Vec<Child>)>;
