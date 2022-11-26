use std::collections::HashMap;
use std::fmt::{Display, self};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Child;
use std::time::Instant;

use crate::config::{ProgramConfig, Config};

pub mod start;
pub mod status;
pub mod stop;
pub mod restart;
pub mod reload;
pub mod shutdown;

#[derive(Debug)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopped,
    Exited,
    Backoff,
    Fatal,
}

impl Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessStatus::Starting => write!(f, "STARTING"),
            ProcessStatus::Running => write!(f, "RUNING"),
            ProcessStatus::Stopped => write!(f, "STOPPED"),
            ProcessStatus::Exited => write!(f, "EXITED"),
            ProcessStatus::Backoff => write!(f, "BACKOFF"),
            ProcessStatus::Fatal => write!(f, "FATAL"),
        }
    }
}

pub struct ProcessInfo {
    pub conf: ProgramConfig,
    pub child: Child,
    pub status: ProcessStatus,
    pub start_time: Instant,
}

impl ProcessInfo {
    fn status_str(&self) -> String {
        //TODO: add padding to improve readbility
        format!("{} {} pid {}, {}s\n",
                self.conf.name,
                self.status,
                self.child.id(),
                //TODO: improve timestamp: HH:mm:ss
                self.start_time.elapsed().as_secs().to_string()
               )
    }
}

pub type ProcessList = HashMap<String, ProcessInfo>;

pub struct Daemon {
    pub conf: Config,
    pub listener: UnixListener,
    pub proc_list: ProcessList,
}

impl Daemon {
    pub fn run(&mut self) {
        loop {
            let (mut stream, _) = self.listener.accept().expect("fail accept");
            let cmd = self.recv_cmd(&mut stream);
            let response = self.run_cmd(cmd.to_string());
            self.send_resp(response, &mut stream);
        }
    }

    pub fn run_cmd(&mut self, line: String) -> String {
        let mut argv: Vec<&str> = line.split_whitespace().collect::<Vec<&str>>();
        //TODO: find a better way to do this
        let mut cmd = "";
        if !argv.is_empty() {
            cmd = argv.remove(0);
        }
        match cmd {
            "start" => start::start(argv, &self.conf, &mut self.proc_list),
            "status" => status::status(argv, &self),
            //"stop" => launch_proces::stop(command, conf),
            //"restart" => launch_proces::restart(command, conf),
            //"reload" => launch_proces::reload(command, conf),
            "shutdown" => shutdown::shutdown(),
            _ => "".to_string(),
        }
    }

    pub fn recv_cmd(&mut self, mut stream: &UnixStream) -> String {
        let mut cmd = String::new();
        stream.read_to_string(&mut cmd).expect("failed to read stream");
        println!("daemon: received command: {}", cmd);
        cmd
    }

    pub fn send_resp(&mut self, response: String, mut stream: &UnixStream) -> () {
        stream.write(response.as_bytes()).expect("failed to write");
        println!("daemon: sending response: {}", response);
    }
}
