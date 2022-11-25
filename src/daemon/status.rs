use super::Daemon;

pub fn status(command: Vec<&str>, daemon: &Daemon) -> String {
    let mut response: String = String::new();
    for (k, v) in &daemon.proc_list {
        response += &format!("{} {} {:?}\n", k, v.child.id(), v.status);
    }
    response
}
