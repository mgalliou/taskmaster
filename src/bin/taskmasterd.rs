extern crate yaml_rust;

use std::collections::HashMap;
use std::io::Read;
use std::os::unix::net::{UnixStream, UnixListener};

use taskmaster::config::{self, Config};
use taskmaster::daemon::{ProcessInfo, start};

fn read_message(listener: &UnixListener) -> String {
    let mut response = String::new();
    let (mut stream, socket) = listener.accept().expect("fail accept");
    stream.read_to_string(&mut response).expect("failed to read stream");
    response
}

fn exec_command(line: String, conf: &Config, proc_list: &mut ProcessInfo) -> () {
    let mut line_split: Vec<&str> = line.split_whitespace().collect::<Vec<&str>>();
    let cmd = line_split.remove(0);
    match cmd {
        "start" => start::start(line_split, conf, proc_list),
        //"status" => launch_proces::status(command, conf),
        //"stop" => launch_proces::stop(command, conf),
        //"restart" => launch_proces::restart(command, conf),
        //"reload" => launch_proces::reload(command, conf),
        //"exit" => launch_proces::exit(command, conf),
        &_ => (),
    }
}

fn main() {
    let conf = config::from_file("cfg/good/cat.yaml".to_string());
    let path = &"/tmp/taskmaster.socket";
    if std::fs::metadata(path).is_ok() {
        println!("A socket is already present. Deleting...");
        std::fs::remove_file(path).expect("could not delete previous socket at {:?}");
    }
    let listener = UnixListener::bind(path).expect("failed to open stream");
    let mut proc_list: ProcessInfo = HashMap::new();
    let mut line: String;
    loop {
        line = read_message(&listener);
        println!("daemon {}", line);
        exec_command(line.to_string(), &conf, &mut proc_list);
    }
}
