use crate::common::comm::send_message;
use crate::config::Config;

use super::Daemon;

pub fn status(command: Vec<&str>, daemon: &Daemon) -> () {
    let mut response: String = String::new();
    for (k, v) in &daemon.proc_list {
        response += &format!("{} {} {:?}\n", k, v.child.id(), v.status);
    }
    print!("{}", response);
}
