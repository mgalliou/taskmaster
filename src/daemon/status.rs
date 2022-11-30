use std::process::{Child, ExitStatus};
use std::time::{Duration, Instant};

use super::{Daemon, ProcessInfo, ProcessStatus};

fn check_time(start: Instant, duration: i64) -> bool {
    if Instant::now() - start > Duration::from_secs(duration.unsigned_abs()) {
        true
    } else {
        false
    }
}

fn get_exit_code(proc: &mut ProcessInfo) -> () {
    match &mut proc.child {
        Some(c) => match c.try_wait() {
            Ok(Some(status)) => {
                proc.exit_status = Some(status);
                proc.exit_time = Some(Instant::now());
            },
            Ok(None) => proc.exit_status = None,
            Err(_) => (),
        },
        None => (),
    }
}

fn get_next_state(
    proc: &mut ProcessInfo,
    ok_some: ProcessStatus,
    ok_none: ProcessStatus,
) -> ProcessStatus {
    get_exit_code(proc);
    match proc.exit_status {
        Some(_) => ok_some,
        None => ok_none,
    }
}

//TODO: check if process starting or stoping should change state based on time
fn check_state(proc: &mut ProcessInfo) -> ProcessStatus {
    match proc.status {
        ProcessStatus::Starting => {
            get_next_state(proc, ProcessStatus::Backoff, ProcessStatus::Starting)
        }
        ProcessStatus::Running => {
            get_next_state(proc, ProcessStatus::Exited, ProcessStatus::Running)
        }
        ProcessStatus::Stopping => {
            get_next_state(proc, ProcessStatus::Stopped, ProcessStatus::Stopping)
        }
        //TODO: exited: check if exitcode it ok and if program needs to restart
        ProcessStatus::Exited => ProcessStatus::Exited,
        //TODO: backoff: check number of retries 
        ProcessStatus::Backoff => ProcessStatus::Backoff,
        ProcessStatus::Unknown => ProcessStatus::Unknown,
        ProcessStatus::Fatal => ProcessStatus::Fatal,
        ProcessStatus::Stopped => ProcessStatus::Stopped,
    }
}

pub fn status(args: Vec<&str>, daemon: &Daemon) -> String {
    let mut response: String = String::new();
    if args.is_empty() {
        for (_, info) in &daemon.proc_list {
            response += &info.status_str();
        }
    } else {
        for prog in args {
            if daemon.proc_list.contains_key(prog) {
                response += &(&daemon.proc_list[prog]).status_str();
            } else {
                response += &format!("process ({}) not found\n", prog)
            }
        }
    }
    response
}
