use std::collections::HashMap;
use std::fmt::{self, Display};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::Child;
use std::time::Instant;

use crate::config::{Config, ProgramConfig};

pub mod reload;
pub mod restart;
pub mod shutdown;
pub mod start;
pub mod status;
pub mod stop;

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
        format!(
            "{:33} {:8} {}\n",
            self.conf.name,
            self.status,
            self.pid_str(),
        )
    }

    fn pid_str(&self) -> String {
        match self.status {
            ProcessStatus::Starting => format!(""),
            ProcessStatus::Running => format!("pid {:8}, {}", self.child.id(), self.uptime_str()),
            //TODO: print exit time when available
            ProcessStatus::Stopped => format!("{:12}", "Not started"),
            ProcessStatus::Exited => format!("{}", self.exittime_str()),
            ProcessStatus::Backoff | ProcessStatus::Fatal => format!("Exited too quickly"),
        }
    }

    fn uptime_str(&self) -> String {
        let s = self.start_time.elapsed().as_secs();
        let m = s / 60;
        let h = s / 3600;
        format!("uptime {:02}:{:02}:{:02}", h, m - (60 * h), s - (3600 * h))
    }

    fn exittime_str(&self) -> String {
        todo!()
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
        stream
            .read_to_string(&mut cmd)
            .expect("failed to read stream");
        println!("daemon: received command: {}", cmd);
        cmd
    }

    pub fn send_resp(&mut self, response: String, mut stream: &UnixStream) -> () {
        stream.write(response.as_bytes()).expect("failed to write");
        println!("daemon: sending response: {}", response);
    }
}
