extern crate yaml_rust;
use std::collections::HashMap;
use std::os::unix::net::UnixListener;
use taskmaster::config;
use taskmaster::daemon::Daemon;

fn main() {
    let path = "taskmaster.socket";
    if std::fs::metadata(path).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(path).expect("could not delete previous socket at {:?}");
    }
    let mut daemon: Daemon = Daemon {
        conf: config::from_file("cfg/good/cat.yaml".to_string()),
        listener: UnixListener::bind(path).expect("failed to open stream"),
        proc_list: HashMap::new(),
    };
    daemon.run();
}
