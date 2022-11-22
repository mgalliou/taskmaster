use std::process::{Command, Child};

use crate::config::{Config,ProgramConfig};

fn status(command: Vec<&str>, conf: &Config) -> () {
}

pub fn start(command: Vec<&str>, conf: &Config) -> Vec<(ProgramConfig, Vec<Child>)> {
    let mut list: Vec<(ProgramConfig, Vec<Child>)>  = Vec::new();
    if command.len() == 1 {
        for (name,prog) in &conf.programs {
            if !(name.is_empty()) {
                // TODO: check if *prog.clone works
                list.push((prog.clone(), launch_process(prog)));
            }
        }
    } else {
        if conf.programs.contains_key(command[1]) {
            list.push((((conf.programs[command[1]]).clone()), launch_process(&conf.programs[command[1]])));
        }
    }
    list
}

fn stop(command: Vec<&str>, conf: &Config) -> () {
}

fn restart(command: Vec<&str>, conf: &Config) -> () {
}

fn reaload(command: Vec<&str>, conf: &Config) -> () {
}

fn exit(command: Vec<&str>, conf: &Config) -> () {
}

fn launch_process(prog: &ProgramConfig) -> Vec<Child> {
    let mut cmd_with_args = prog.cmd.split_whitespace();
    let cmd = match cmd_with_args.nth(0) {
        Some(n) => n,
        None => panic!("command not found")
    };
    let args = cmd_with_args.skip(1);
    let mut child = Vec::new();
    for _i in 0..prog.numprocs {
        child.push(Command::new(cmd)
            .args(args.clone())
            .spawn()
            .expect("failed to execute child")
    )};
    child
}
