use std::ffi::OsStr;
use std::process::{Child, Command, Stdio};
use crate::config::{Config, ProgramConfig};
extern crate libc;

fn exec_cmd<I, S>(cmd_name: &str, args: I, prog: &ProgramConfig) -> Child 
where 
I : IntoIterator<Item = S>,
S : AsRef<OsStr>,
{
    let cmd = Command::new(cmd_name)
        .args(args)
        .stdout(prog.open_stdout())
        .stderr(prog.open_stderr())
        .stdin(Stdio::null())
        .current_dir(prog.workingdir.clone())
        .spawn()
        .expect("failed to execute child");
    cmd
}

fn start_program(prog: &ProgramConfig) -> Vec<Child> {
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
        let cmd = exec_cmd(cmd_name, args.clone(), prog);
        childs.push(cmd)
    }
    unsafe {
        libc::umask(mode);
    }
    childs
}

pub fn start(cmd: Vec<&str>, conf: &Config) -> super::ProcessInfo {
    let mut list: Vec<(ProgramConfig, Vec<Child>)> = Vec::new();
    if cmd.len() == 1 {
        for (name, prog) in &conf.programs {
            if !(name.is_empty()) {
                // TODO: check if *prog.clone works
                list.push((prog.clone(), start_program(prog)));
            }
        }
    } else {
        if conf.programs.contains_key(cmd[1]) {
            list.push((
                ((conf.programs[cmd[1]]).clone()),
                start_program(&conf.programs[cmd[1]]),
            ));
        }
    }
    list
}
