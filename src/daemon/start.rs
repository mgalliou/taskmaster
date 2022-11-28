extern crate libc;
use super::{ProcessInfo, ProcessList, ProcessStatus, status, Daemon};
use crate::config::{Config, ProgramConfig};
use std::ffi::OsStr;
use std::io;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

fn exec_cmd<I, S>(cmd_name: &str, args: I, prog_conf: &ProgramConfig) -> io::Result<Child>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mode = unsafe { libc::umask(prog_conf.umask) };
    let ret = Command::new(cmd_name)
        .args(args)
        .stdout(prog_conf.open_stdout())
        .stderr(prog_conf.open_stderr())
        .stdin(Stdio::null())
        .current_dir(prog_conf.workingdir.clone())
        .spawn();
    unsafe {
        libc::umask(mode);
    }
    ret
}

fn start_program(name: String, proc: &mut ProcessInfo) -> String {
    let mut argv = proc.conf.cmd.split_whitespace();
    let cmd_name = match argv.nth(0) {
        Some(n) => n,
        None => "",
    };
    let args = argv.clone().skip(1);
    let cmd = exec_cmd(cmd_name, args, &proc.conf);
    if cmd.is_ok() {
        proc.child = cmd.ok();
        proc.status = ProcessStatus::Starting;
        proc.start_time = Instant::now();
        proc.start_nb += 1;
        format!("{}: started\n", name)
    } else {
        proc.status = ProcessStatus::Stopped;
        proc.start_time = Instant::now();
        format!("{}: not started\n", name)
    }
}

pub fn start(line: Vec<&str>, daemon: &mut Daemon) -> String {
    let mut response: String = String::new();
    if line.len() > 0 {
        for program in line {
            //TODO:check get_mut return
            if daemon.proc_list.contains_key(program) {
                response += &start_program(program.to_string(), &mut daemon.proc_list.get_mut(program).unwrap());
            }
        }
    } else {
        for (program, proc_info) in &mut daemon.proc_list {
            response += &start_program(program.to_string(), proc_info);
        }
    }
    response
}
