use super::Daemon;

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
