use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};
use std::time::Instant;
use crate::config::{Config, ProgramConfig};

use super::{CommandResult, ProcessList, ProcessInfo, ProcessStatus};
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

fn start_program(name: &str, prog_conf: &ProgramConfig, proc_list: &mut ProcessList) -> String {
    let mut argv = prog_conf.cmd.split_whitespace();
    let cmd_name = match argv.nth(0) {
        Some(n) => n,
        None => "",
    };
    let args = argv.clone().skip(1);
    let cmd = exec_cmd(cmd_name, args, &prog_conf);
    if cmd.is_ok() {
        let proc_info = ProcessInfo {
            conf: prog_conf.clone(),
            child: cmd.ok().unwrap(),
            status: ProcessStatus::Starting,
            start_time: Instant::now(),

        };
        (*proc_list).entry(name.to_string()).or_insert(proc_info);
        format!("{}: ok", name)
    }else {
        format!("{}: not ok", name)
    }
}

pub fn start(line:Vec<&str>, conf: &Config, proc_list: &mut ProcessList) -> String {
    let mut response: String = String::new();
    if line.len() > 0 {
        for program in line {
            response += &start_program(program, &conf.programs[program], proc_list);
        }
    } else {
        for (program, program_config) in &conf.programs {
            response += &start_program(program.as_str(), program_config, proc_list);
        }
    }
    response
}
