use std::time::Instant;

use nix::{sys::signal, unistd::Pid};

use super::{Daemon, ProcessInfo, ProcessStatus};

#[allow(unused_must_use)]
fn stop_program(name: String, proc: &mut ProcessInfo) -> String {
    match proc.child.as_mut() {
        Some(c) => {
            signal::kill(Pid::from_raw(c.id() as i32), proc.conf.stopsignal);
            proc.status = ProcessStatus::Stopping;
            proc.stop_time = Some(Instant::now());
            format!("{}: stopped\n", name)
        }
        None => format!("{}: not running\n", name),
    }
}

pub fn stop(args: Vec<&str>, daemon: &mut Daemon) -> String {
    let mut response: String = String::new();
    if args.len() > 0 {
        for program in args {
            if daemon.proc_list.contains_key(program) {
                response += &stop_program(program.to_string(), &mut daemon.proc_list.get_mut(program).unwrap());
            } else {
                response += &format!("{}: ERROR (no such process)", program)
            }
        }
    } else {
        for (program, proc_info) in &mut daemon.proc_list {
            response += &stop_program(program.to_string(), proc_info);
        }
    }
    response
}
