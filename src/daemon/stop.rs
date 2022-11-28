use super::{Daemon, ProcessInfo, ProcessStatus};

fn stop_program(name: String, proc: &mut ProcessInfo) -> String {
    match proc.child.as_mut() {
        //TODO: should send stopsignal first, wait for stoptime then kill
        Some(c) => match c.kill() {
            Ok(()) => {
                proc.status = ProcessStatus::Stopped;
                format!("{}: stopped\n", name)
            }
            Err(_) => format!("{}: failed to stop process\n", name),
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
