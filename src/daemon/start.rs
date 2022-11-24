use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};
use crate::config::{Config, ProgramConfig};

use super::{CommandResult, ProcessInfo};
extern crate libc;

fn exec_cmd<I, S>(cmd_name: &str, args: I, prog_conf: &ProgramConfig) -> io::Result<Child> 
where 
I : IntoIterator<Item = S>,
S : AsRef<OsStr>,
{
    let mode = unsafe {libc::umask(prog_conf.umask)};
    let ret = Command::new(cmd_name)
        .args(args)
        .stdout(prog_conf.open_stdout())
        .stderr(prog_conf.open_stderr())
        .stdin(Stdio::null())
        .current_dir(prog_conf.workingdir.clone())
        .spawn();
    unsafe {libc::umask(mode);}
    ret
}

fn start_program(name: String, prog_conf: ProgramConfig, proc_list: &mut ProcessInfo) -> CommandResult {
    let mut argv = prog_conf.cmd.split_whitespace();
    let mut res : bool = false;
    let cmd_name = match argv.nth(0) {
        Some(n) => n,
        None => "",
    };
    let args = argv.clone().skip(1);
    let cmd = exec_cmd(cmd_name, args, &prog_conf);
    if cmd.is_ok() {
        res = true;
        (*proc_list).entry(name.clone()).or_insert((prog_conf.clone(), cmd.ok().unwrap()));
    };
    CommandResult::new(res, name, argv.map(|s| s.to_string()).collect::<Vec<String>>(), String::new())
}

pub fn start_numprocs(name: &str, prog_conf: &ProgramConfig, proc_list: &mut ProcessInfo) -> () {
    for i in 0..prog_conf.numprocs {
        let res = start_program(name.to_string() + &i.to_string(), prog_conf.clone(),proc_list);
        if res.ok == false {
        }
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
