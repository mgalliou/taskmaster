use super::Daemon;

pub fn update_status(daemon: &mut Daemon) -> () {
    for (_, proc) in &mut daemon.proc_list {
        match &mut proc.child {
            Some(c) => match c.try_wait() {
                Ok(Some(status)) => println!("exited with: {status}"),
                Ok(None) => {}
                Err(e) => println!("error attempting to wait: {e}"),
            },
            None => println!("no child"),
        }
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
