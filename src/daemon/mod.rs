use std::collections::HashMap;
use std::process::Child;

use crate::config::ProgramConfig;

pub mod start;
pub mod status;
pub mod stop;
pub mod restart;
pub mod reload;
pub mod exit;

pub type ProcessInfo = HashMap<String, (ProgramConfig, Child)>;
pub struct CommandResult {
    ok: bool,
    command: String,
    args: Vec<String>,
    message: String
}

impl CommandResult {
    pub fn new(ok: bool, command: String, args: Vec<String>, message: String) -> Self { Self { ok, command, args, message } }
}
