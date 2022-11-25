use std::collections::HashMap;
use std::os::unix::net::UnixListener;
use std::process::Child;

use crate::config::{ProgramConfig, Config};

pub mod start;
pub mod status;
pub mod stop;
pub mod restart;
pub mod reload;
pub mod exit;

pub type ProcessInfo = HashMap<String, (ProgramConfig, Child)>;

pub struct Daemon {
    pub conf: Config,
    pub listener: UnixListener,
    pub proc_list: ProcessInfo,
}

impl Daemon {
    pub fn exec_command(&mut self, line: String) -> () {
        let mut line_split: Vec<&str> = line.split_whitespace().collect::<Vec<&str>>();
        let cmd = line_split.remove(0);
        match cmd {
            "start" => start::start(line_split, &self.conf, &mut self.proc_list),
            //"status" => launch_proces::status(command, conf),
            //"stop" => launch_proces::stop(command, conf),
            //"restart" => launch_proces::restart(command, conf),
            //"reload" => launch_proces::reload(command, conf),
            //"exit" => launch_proces::exit(command, conf),
            &_ => (),
        }
    }

    pub fn proc_list_mut(&mut self) -> &mut ProcessInfo {
        &mut self.proc_list
    }
}


pub struct CommandResult {
    ok: bool,
    command: String,
    args: Vec<String>,
    message: String
}

impl CommandResult {
    pub fn new(ok: bool, command: String, args: Vec<String>, message: String) -> Self { Self { ok, command, args, message } }
}
