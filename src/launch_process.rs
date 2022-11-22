use std::fs::File;
use std::process::{Command, Child, Stdio};

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
    let cmd_name = match cmd_with_args.nth(0) {
        Some(n) => n,
        None => panic!("command not found")
    };
    let args = cmd_with_args.skip(1);
    let mut child: Vec<Child> = Vec::new();
    // TODO: check file error handling
    for _i in 0..prog.numprocs {
        //TODO: refactor this DAEMON sah
        child.push(if prog.stdout.is_empty() && prog.stderr.is_empty() {
            Command::new(cmd_name)
                .args(args.clone())
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute child")
        } else if prog.stdout.is_empty() && !prog.stderr.is_empty() {
            let f = File::create(&prog.stderr);
            Command::new(cmd_name)
                .args(args.clone())
                .stderr(f.unwrap())
                .stdout(Stdio::null())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute child")
        } else if !prog.stdout.is_empty() && !prog.stderr.is_empty() {
            let f = File::create(&prog.stderr);
            let g = File::create(&prog.stdout);
            Command::new(cmd_name)
                .args(args.clone())
                .stderr(f.unwrap())
                .stdout(g.unwrap())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute child")
        } else {
            let g = File::create(&prog.stdout);
            Command::new(cmd_name)
                .args(args.clone())
                .stderr(Stdio::null())
                .stdout(g.unwrap())
                .stdin(Stdio::null())
                .spawn()
                .expect("failed to execute child")
        });
    };
    child
}
