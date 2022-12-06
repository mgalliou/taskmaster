use core::fmt;
use nix::sys::signal::Signal;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::process::Stdio;
use std::str::FromStr;
use yaml_rust::{Yaml, YamlLoader};

const DEFAULT_NUMPROCS: i64 = 1;
const DEFAULT_UMASK: u32 = 0o022;
const DEFAULT_CWD: &str = ".";
const DEFAULT_AUTOSTART: bool = true;
const DEFAULT_AUTORESTART: RestartPolicy = RestartPolicy::Unexpected;
const DEFAULT_EXITCODES: [i64; 1] = [0];
const DEFAULT_STARTRETRIES: i64 = 3;
const DEFAULT_STARTTIME: i64 = 10;
const DEFAULT_STOPSIGNAL: &str = "TERM";
const DEFAULT_STOPTIME: i64 = 10;
const DEFAULT_STDOUT: &str = "";
const DEFAULT_STDERR: &str = "";

#[derive(Debug)]
pub struct ConfigError {
    details: String,
}

impl ConfigError {
    fn new(msg: &str) -> ConfigError {
        ConfigError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RestartPolicy {
    Always,
    Never,
    Unexpected,
}

impl FromStr for RestartPolicy {
    //TODO: Use a better error type
    type Err = ();

    fn from_str(s: &str) -> Result<RestartPolicy, Self::Err> {
        match s {
            "always" => Ok(RestartPolicy::Always),
            "never" => Ok(RestartPolicy::Never),
            "unexpected" => Ok(RestartPolicy::Unexpected),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramConfig {
    pub name: String,
    pub cmd: String,
    pub numprocs: i64,
    pub umask: u32,
    pub workingdir: String,
    pub autostart: bool,
    pub autorestart: RestartPolicy,
    pub exitcodes: Vec<i64>,
    pub stdout: String,
    pub stderr: String,
    pub startretries: i64,
    pub starttime: i64,
    pub stopsignal: Signal,
    pub stoptime: i64,
    pub env: HashMap<String, String>,
}

impl ProgramConfig {
    fn from_yaml(name: String, conf: &Yaml) -> Result<ProgramConfig, ConfigError> {
        Ok(ProgramConfig {
            name,
            //TODO: return error with missing cmd field
            cmd: get_str_field(conf, "cmd", "")?,
            numprocs: get_num_field(conf, "numprocs", DEFAULT_NUMPROCS)?,
            umask: get_umask(conf, "umask")?,
            //TODO: test behavior with no workingdir
            workingdir: get_str_field(conf, "workingdir", DEFAULT_CWD)?,
            autostart: get_bool_field(conf, "autostart", DEFAULT_AUTOSTART)?,
            autorestart: get_autorestart(conf, "autorestart")?,
            exitcodes: get_num_vec_field(conf, "exitcodes", DEFAULT_EXITCODES.to_vec())?,
            startretries: get_num_field(conf, "startretries", DEFAULT_STARTRETRIES)?,
            starttime: get_num_field(conf, "starttime", DEFAULT_STARTTIME)?,
            stopsignal: get_stop_signal(conf)?,
            stoptime: get_num_field(conf, "stoptime", DEFAULT_STOPTIME)?,
            //TODO: define default behavior with no stdout or stderr
            stdout: get_str_field(conf, "stdout", DEFAULT_STDOUT)?,
            stderr: get_str_field(conf, "stderr", DEFAULT_STDERR)?,
            env: get_hash_str_field(conf, "env", HashMap::new())?,
        })
    }

    pub fn open_stdout(&self) -> Stdio {
        if self.stdout.is_empty() {
            Stdio::null()
        } else {
            match File::create(&self.stdout) {
                Ok(f) => Stdio::from(f),
                //TODO: log file opening failure
                Err(_) => Stdio::null(),
            }
        }
    }

    pub fn open_stderr(&self) -> Stdio {
        if self.stderr.is_empty() {
            Stdio::null()
        } else {
            match File::create(&self.stderr) {
                Ok(f) => Stdio::from(f),
                //TODO: log file opening failure
                Err(_) => Stdio::null(),
            }
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub programs: HashMap<String, ProgramConfig>,
}

impl Config {
    fn from_yaml(yaml: &Yaml) -> Result<Config, ConfigError> {
        let mut programs: HashMap<String, ProgramConfig> = HashMap::new();
        let yprog = match yaml["programs"].as_hash() {
            Some(y) => Ok(y),
            None => Err(ConfigError::new("no program field found")),
        }?;
        for (yname, yconf) in yprog.into_iter() {
            let numprocs = get_num_field(yconf, "numprocs", 1)?;
            let base_name = match yname.as_str() {
                Some(n) => Ok(n),
                None => Err(ConfigError::new("program name is not a string")),
            }?;
            for i in 0..numprocs {
                let name = gen_name(numprocs, base_name, i);
                let conf = ProgramConfig::from_yaml(name.clone(), yconf)?;
                programs.insert(name.clone(), conf);
            }
        }
        Ok(Config { programs })
    }

    pub fn from_str(str: String) -> Result<Config, ConfigError> {
        match YamlLoader::load_from_str(&str) {
            Ok(yaml) => Config::from_yaml(&yaml[0]),
            Err(e) => Err(ConfigError::new(&format!(
                "error scanning config file: {}",
                e.to_string()
            ))),
        }
    }

    pub fn from_file(path: &str) -> Result<Config, ConfigError> {
        let yaml_str = match fs::read_to_string(path) {
            Ok(f) => Ok(f),
            Err(e) => Err(ConfigError::new(&format!(
                "Failed to read config file: {}",
                e
            ))),
        }?;
        match Config::from_str(yaml_str) {
            Ok(c) => Ok(c),
            Err(e) => Err(ConfigError::new(&format!("Failed to parse yaml: {}", e))),
        }

    }
}

fn get_autorestart(prog: &Yaml, field: &str) -> Result<RestartPolicy, ConfigError> {
    let f = &prog[field];
    if f.is_badvalue() {
        return Ok(DEFAULT_AUTORESTART);
    }
    let s = f.as_str();
    if s.is_some() {
        match RestartPolicy::from_str(s.unwrap()) {
            Ok(rp) => return Ok(rp),
            Err(_) => {},
        }
    }
    Err(ConfigError::new(&format!("invalid value for field: {}", field)))
}

fn get_bool_field(prog: &Yaml, field: &str, default: bool) -> Result<bool, ConfigError> {
    match prog[field] {
        Yaml::Boolean(b) => Ok(b),
        Yaml::BadValue => Ok(default),
        _ => Err(ConfigError::new(&format!("field is not a boolean: {}", field))),
    }
}

fn get_umask(prog: &Yaml, field: &str) -> Result<u32, ConfigError> {
    if prog[field].is_badvalue() {
        return Ok(DEFAULT_UMASK)
    };
    match prog[field].as_i64() {
        Some(s) => match u32::from_str_radix(&s.to_string(), 8) {
            Ok(n) => Ok(n),
            Err(_) => Err(ConfigError::new(&format!(
                "failed to convert umask: {}",
                field
            ))),
        },
        None => Err(ConfigError::new(&format!(
            "field not found or not a number: {}",
            field
        ))),
    }
}

fn get_num_field(prog: &Yaml, field: &str, default: i64) -> Result<i64, ConfigError> {
    match prog[field] {
        Yaml::Integer(n) => Ok(n),
        Yaml::BadValue => Ok(default),
        _ => Err(ConfigError::new(&format!("invalid value for field: {}", field)))
    }
}

fn get_num_vec_field(prog: &Yaml, field: &str, default: Vec<i64>) -> Result<Vec<i64>, ConfigError> {
    let f = match &prog[field] {
        Yaml::BadValue => return Ok(default),
        Yaml::Array(v) => Ok(v),
        _ => Err(ConfigError::new(&format!("field not found: {}", field))),
    }?;
    f.into_iter()
        .map(|n| match n.as_i64() {
            Some(n) => Ok(n),
            None => Err(ConfigError::new(&format!(
                        "invalid value for field: {}",
                        field
                        ))),
        })
        .collect()
}

fn get_hash_str_field(prog: &Yaml, field: &str, default: HashMap<String, String>) -> Result<HashMap<String, String>, ConfigError> {
    let f = match &prog[field] {
        Yaml::Hash(h) => Ok(h.clone()),
        Yaml::BadValue => return Ok(default),
        _ => return Err(ConfigError::new(&format!(
                    "invalid value for field: {}",
                    field
                    ))),
    }?;
    f.into_iter()
        .map(|(k, v)| {
            let new_k = match k.into_string() {
                Some(k) => k,
                None => {
                    return Err(ConfigError::new(&format!(
                                "invalid key in field: {}",
                                field
                                )))
                }
            };
            let new_v = match v.into_string() {
                Some(v) => v,
                None => {
                    return Err(ConfigError::new(&format!(
                                "invalid key in field: {}",
                                field
                                )))
                }
            };
            Ok((new_k, new_v))
        })
    .collect()
}

fn get_str_field(prog: &Yaml, field: &str, default: &str) -> Result<String, ConfigError> {
    match &prog[field] {
        Yaml::BadValue => Ok(default.to_string()),
        Yaml::String(s) => Ok(s.to_string()),
        _ => Err(ConfigError::new(&format!("field is not a string: {}", field))),
    }
}

fn gen_name(numprocs: i64, base_name: &str, i: i64) -> String {
    if numprocs == 1 {
        base_name.to_string()
    } else {
        base_name.to_string() + &i.to_string()
    }
}

fn get_stop_signal(prog: &Yaml) -> Result<Signal, ConfigError> {
    let ss = get_str_field(prog, "stopsignal", DEFAULT_STOPSIGNAL)?;
    match ("SIG".to_owned() + &ss).parse::<Signal>() {
        Ok(s) => Ok(s),
        Err(_) => Ok(Signal::SIGTERM),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{self, Config};
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let c = Config::from_file("cfg/good/cat.yaml").unwrap();
        assert_eq!(c.programs["cat"].cmd, "/bin/cat");
        assert_eq!(c.programs["cat"].numprocs, 1);
        assert_eq!(c.programs["cat"].umask, 0o022);
        assert_eq!(c.programs["cat"].workingdir, "/");
        assert_eq!(c.programs["cat"].autostart, true);
        assert_eq!(
            c.programs["cat"].autorestart,
            config::RestartPolicy::Unexpected
        );
        assert_eq!(c.programs["cat"].exitcodes, vec![0, 2]);
        assert_eq!(c.programs["cat"].startretries, 3);
        assert_eq!(c.programs["cat"].starttime, 5);
        assert_eq!(c.programs["cat"].stopsignal.as_str(), "SIGTERM");
        assert_eq!(c.programs["cat"].stoptime, 10);
        assert_eq!(c.programs["cat"].stdout, "/tmp/cat.stdout");
        assert_eq!(c.programs["cat"].stderr, "/tmp/cat.stderr");
        assert_eq!(
            c.programs["cat"].env,
            HashMap::from([
                ("STARTED_BY".to_string(), "taskmaster".to_string()),
                ("ANSWER".to_string(), "42".to_string())
            ])
        );
    }
}
