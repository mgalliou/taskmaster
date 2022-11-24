use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};
use crate::config::{Config, ProgramConfig};

use super::{CommandResult, ProcessInfo};
extern crate libc;

fn exec_cmd<I, S>(cmd_name: &str, args: I, prog: &ProgramConfig) -> io::Result<Child> 
where 
I : IntoIterator<Item = S>,
S : AsRef<OsStr>,
{
    Command::new(cmd_name)
        .args(args)
        .stdout(prog.open_stdout())
        .stderr(prog.open_stderr())
        .stdin(Stdio::null())
        .current_dir(prog.workingdir.clone())
        .spawn()
}
fn start_program(name: String, prog_conf: ProgramConfig, proc_list: &mut ProcessInfo) -> () {
    let mut argv = prog_conf.cmd.split_whitespace();
    let cmd_name = match argv.nth(0) {
        Some(n) => n,
        //TODO: should not panic
        None => panic!("command not found"),
    };
    let args = argv.skip(1);
    let mode = unsafe { libc::umask(prog_conf.umask) };
    let cmd = exec_cmd(cmd_name, args.clone(), &prog_conf);
    if cmd.is_ok() {
        (*proc_list).entry(name).or_insert((prog_conf, cmd.ok().unwrap()));
    };
    unsafe {
        libc::umask(mode);
    }
}

pub fn start_numprocs(name: &str, prog_conf: &ProgramConfig, proc_list: &mut ProcessInfo) -> () {
    for i in 0..prog_conf.numprocs {
        start_program(name.to_string() + &i.to_string(), prog_conf.clone(),proc_list);
    }
}

pub fn start(line:Vec<&str>, conf: &Config, proc_list: &mut ProcessInfo) -> () {
    if line.len() > 0 {
        for program in line {
            start_numprocs(program, &conf.programs[program], proc_list);
        }
    } else {
        for (program, program_config) in &conf.programs {
            start_numprocs(&program, &program_config, proc_list);
        }
    }
}
