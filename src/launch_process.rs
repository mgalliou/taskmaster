use std::process::{Child, Command, Stdio};
extern crate libc;

use crate::config::{Config, ProgramConfig};

pub type ProcessInfo = Vec<(ProgramConfig, Vec<Child>)>;

pub fn start(cmd: Vec<&str>, conf: &Config) -> ProcessInfo {
    let mut list: ProcessInfo = Vec::new();
    if cmd.len() == 1 {
        for (name, prog) in &conf.programs {
            if !(name.is_empty()) {
                // TODO: check if *prog.clone works
                list.push((prog.clone(), launch_process(prog)));
            }
        }
    } else {
        if conf.programs.contains_key(cmd[1]) {
            list.push((
                ((conf.programs[cmd[1]]).clone()),
                launch_process(&conf.programs[cmd[1]]),
            ));
        }
    }
    list
}

fn status(command: Vec<&str>, conf: &Config) -> () {}

fn stop(command: Vec<&str>, conf: &Config) -> () {}

fn restart(command: Vec<&str>, conf: &Config) -> () {}

fn reaload(command: Vec<&str>, conf: &Config) -> () {}

fn exit(command: Vec<&str>, conf: &Config) -> () {}

fn launch_process(prog: &ProgramConfig) -> Vec<Child> {
    let mut argv = prog.cmd.split_whitespace();
    let cmd_name = match argv.nth(0) {
        Some(n) => n,
        //TODO: should not panic
        None => panic!("command not found"),
    };
    let args = argv.skip(1);
    let mut childs: Vec<Child> = Vec::new();
    let mode = unsafe { libc::umask(prog.umask) };
    for _i in 0..prog.numprocs {
        let cmd = Command::new(cmd_name)
            .args(args.clone())
            .stdout(prog.open_stdout())
            .stderr(prog.open_stderr())
            .stdin(Stdio::null())
            .current_dir(prog.workingdir.clone())
            .spawn()
            .expect("failed to execute child");
        childs.push(cmd)
    }
    unsafe {
        libc::umask(mode);
    }
    childs
}
