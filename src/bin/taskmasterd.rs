extern crate yaml_rust;
use std::collections::HashMap;
use std::env;
use std::os::unix::net::UnixListener;
use taskmaster::cfg::{ConfigError, Config};
use taskmaster::daemon::Daemon;

fn main() -> Result<(), ConfigError> {
    //TODO: get config path from argv
    let args = env::args().collect::<Vec<String>>();
    let cfg_path = match args.get(1) {
        Some(a) => a,
        None => "taskmaster.yaml",
    };
    let path = "taskmaster.socket";
    if std::fs::metadata(path).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(path).expect("could not delete previous socket at {:?}");
    }
    let mut daemon: Daemon = Daemon {
        conf: Config::from_file(cfg_path)?,
        listener: UnixListener::bind(path).expect("failed to open stream"),
        proc_list: HashMap::new(),
    };
    Ok(daemon.run())
    //TODO: test behavior with invalid config file
}
