use std::process::Command;

use crate::config::Config;
use crate::config::ProgramConfig;

fn status() -> () {
}

pub fn start(command: Vec<&str>, conf: &Config) -> () {
    if command.len() == 1 {
        for (name,prog) in &conf.programs {
            if !(name.is_empty()) {
                launch_process(prog);
            }
        }
    } else {
        if conf.programs.contains_key(command[1]) {
            launch_process(&conf.programs[command[1]]);
        }
    }
}

fn stop() -> () {
}

fn restart() -> () {
}

fn reaload() -> () {
}

fn exit() -> () {
}

fn launch_process(prog: &ProgramConfig) -> () {
    Command::new(&prog.cmd);
}
