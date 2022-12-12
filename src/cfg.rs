use core::fmt;
use nix::sys::signal::Signal;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::process::Stdio;
use std::str::FromStr;
use yaml_rust::{Yaml, YamlLoader};

const DFLT_NUMPROCS: i64 = 1;
const DFLT_UMASK: u32 = 0o022;
const DFLT_CWD: Option<String> = None;
const DFLT_AUTOSTART: bool = true;
const DFLT_AUTORESTART: RestartPolicy = RestartPolicy::Unexpected;
const DFLT_EXITCODES: [i64; 1] = [0];
const DFLT_STARTRETRIES: i64 = 3;
const DFLT_STARTTIME: i64 = 10;
const DFLT_STOPSIGNAL: &str = "TERM";
const DFLT_STOPTIME: i64 = 10;
const DFLT_STDOUT: &str = "AUTO";
const DFLT_STDERR: &str = "AUTO";

#[derive(Debug)]
pub struct Config {
    pub programs: HashMap<String, ProgramConfig>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, ConfigError> {
        let yaml_str = match fs::read_to_string(path) {
            Ok(f) => Ok(f),
            Err(e) => Err(ConfigError::from_unreadable_file(e)),
        }?;
        match Config::from_str(&yaml_str) {
            Ok(c) => Ok(c),
            Err(e) => Err(ConfigError::from_invalid_cfg_file(e)),
        }
    }

    pub fn from_str(str: &str) -> Result<Config, ConfigError> {
        match YamlLoader::load_from_str(str) {
            Ok(yaml) => Config::from_yaml(&yaml[0]),
            Err(e) => Err(ConfigError::from_invalid_yaml(e)),
        }
    }

    fn from_yaml(yaml: &Yaml) -> Result<Config, ConfigError> {
        let mut programs: HashMap<String, ProgramConfig> = HashMap::new();
        let yprog = match yaml["programs"].as_hash() {
            Some(y) => Ok(y),
            None => Err(ConfigError::new("no programs field found")),
        }?;
        for (yname, yconf) in yprog.into_iter() {
            let numprocs = get_num_field(yconf, "numprocs", 1)?;
            let base_name = match yname.as_str() {
                Some(n) => Ok(n),
                None => Err(ConfigError::new("program name is not a string")),
            }?;
            for i in 0..numprocs {
                let name = gen_name(numprocs, base_name, i);
                let conf = ProgramConfig::from_yaml(yconf, name.clone())?;
                programs.insert(name.clone(), conf);
            }
        }
        Ok(Config { programs })
    }
}

#[derive(Debug, Clone)]
pub struct ProgramConfig {
    pub name: String,
    pub cmd: String,
    pub numprocs: i64,
    pub umask: u32,
    pub workingdir: Option<String>,
    pub autostart: bool,
    pub autorestart: RestartPolicy,
    pub exitcodes: Vec<i64>,
    pub stdout: LogPath,
    pub stderr: LogPath,
    pub startretries: i64,
    pub starttime: i64,
    pub stopsignal: Signal,
    pub stoptime: i64,
    pub env: HashMap<String, String>,
}

impl ProgramConfig {
    fn from_yaml(yaml: &Yaml, name: String) -> Result<ProgramConfig, ConfigError> {
        Ok(ProgramConfig {
            name,
            cmd: get_str_field(yaml, "cmd", None)?,
            numprocs: get_num_field(yaml, "numprocs", DFLT_NUMPROCS)?,
            umask: get_umask(yaml, "umask")?,
            workingdir: get_opt_str_field(yaml, "workingdir", DFLT_CWD)?,
            autostart: get_bool_field(yaml, "autostart", DFLT_AUTOSTART)?,
            autorestart: get_autorestart(yaml, "autorestart")?,
            exitcodes: get_num_vec_field(yaml, "exitcodes", DFLT_EXITCODES.to_vec())?,
            startretries: get_num_field(yaml, "startretries", DFLT_STARTRETRIES)?,
            starttime: get_num_field(yaml, "starttime", DFLT_STARTTIME)?,
            stopsignal: get_signal_field(yaml, "stopsignal", DFLT_STOPSIGNAL)?,
            stoptime: get_num_field(yaml, "stoptime", DFLT_STOPTIME)?,
            stdout: get_log_path_field(yaml, "stdout", DFLT_STDOUT)?,
            stderr: get_log_path_field(yaml, "stderr", DFLT_STDERR)?,
            env: get_hash_str_field(yaml, "env", HashMap::new())?,
        })
    }

    pub fn open_stdout(&self) -> Stdio {
        match &self.stdout {
            LogPath::Path(s) => match File::create(s) {
                Ok(f) => Stdio::from(f),
                //TODO: log file opening failure
                Err(_) => Stdio::null(),
            }
            LogPath::Auto => match File::create(format!("/tmp/tackmasterd/{}.stdout.log", self.name)) {
                Ok(f) => Stdio::from(f),
                //TODO: log file opening failure
                Err(_) => Stdio::null(),
            }
            LogPath::Non => Stdio::null()
        }
    }

    pub fn open_stderr(&self) -> Stdio {
        match &self.stderr {
            LogPath::Path(s) => match File::create(s) {
                Ok(f) => Stdio::from(f),
                //TODO: log file opening failure
                Err(_) => Stdio::null(),
            }
            LogPath::Auto => match File::create(format!("/tmp/tackmasterd/{}.stderr.log", self.name)) {
                Ok(f) => Stdio::from(f),
                //TODO: log file opening failure
                Err(_) => Stdio::null(),
            }
            LogPath::Non => Stdio::null()
        }
    }
}

fn gen_name(numprocs: i64, base_name: &str, i: i64) -> String {
    if numprocs == 1 {
        base_name.to_string()
    } else {
        base_name.to_string() + &i.to_string()
    }
}

fn get_str_field(prog: &Yaml, field: &str, default: Option<&str>) -> Result<String, ConfigError> {
    match (&prog[field], default) {
        (Yaml::BadValue, Some(d)) => Ok(d.to_string()),
        (Yaml::BadValue, None) => Err(ConfigError::new(&format!("missing value for field: {}", field))),
        (Yaml::String(s), _) => Ok(s.to_string()),
        (_, _) => Err(ConfigError::from_not_string(field)),
    }
}

fn get_num_field(prog: &Yaml, field: &str, default: i64) -> Result<i64, ConfigError> {
    match prog[field] {
        Yaml::BadValue => Ok(default),
        Yaml::Integer(n) => Ok(n),
        _ => Err(ConfigError::from_not_number(field))
    }
}

fn get_umask(prog: &Yaml, field: &str) -> Result<u32, ConfigError> {
    match prog[field] {
        Yaml::BadValue => Ok(DFLT_UMASK),
        Yaml::Integer(n) => Ok(u32::from_str_radix(&n.to_string(), 8)
                               .expect("field should be valid")),
        _ => Err(ConfigError::from_not_number(field)),
    }
}

fn get_autorestart(prog: &Yaml, field: &str) -> Result<RestartPolicy, ConfigError> {
    let f = &prog[field];
    if f.is_badvalue() {
        return Ok(DFLT_AUTORESTART);
    }
    if let Some(s) = f.as_str() {
        if let Ok(rp) = RestartPolicy::from_str(s) {
            return Ok(rp);
        }
    }
    Err(ConfigError::from_invalid_value(field))
}

fn get_opt_str_field(prog: &Yaml, field: &str, default: Option<String> ) -> Result<Option<String>, ConfigError> {
    match (&prog[field], default) {
        (Yaml::BadValue, Some(d)) => Ok(Some(d.to_string())),
        (Yaml::BadValue, None) => Ok(None),
        (Yaml::String(s), _) => Ok(Some(s.to_string())),
        (_, _) => Err(ConfigError::from_not_string(field)),
    }
}

fn get_bool_field(prog: &Yaml, field: &str, default: bool) -> Result<bool, ConfigError> {
    match prog[field] {
        Yaml::BadValue => Ok(default),
        Yaml::Boolean(b) => Ok(b),
        _ => Err(ConfigError::from_not_bool(field)),
    }
}

fn get_num_vec_field(prog: &Yaml, field: &str, default: Vec<i64>) -> Result<Vec<i64>, ConfigError> {
    let f = match &prog[field] {
        Yaml::BadValue => return Ok(default),
        Yaml::Array(v) => Ok(v),
        _ => Err(ConfigError::from_not_array(field)),
    }?;
    f.into_iter()
        .map(|n| match n.as_i64() {
            Some(n) => Ok(n),
            None => Err(ConfigError::from_array_value_not_nbr(field)),
        })
    .collect()
}

fn get_signal_field(prog: &Yaml, field: &str, default: &str) -> Result<Signal, ConfigError> {
    let ss = get_str_field(prog, field, Some(default))?;
    match ("SIG".to_owned() + &ss).parse::<Signal>() {
        Ok(s) => Ok(s),
        Err(_) => Err(ConfigError::from_invalid_value(field)),
    }
}

fn get_log_path_field(prog: &Yaml, field: &str, default: &str) -> Result<LogPath, ConfigError> {
    let lp = match get_str_field(prog, field, Some(default)) {
        Ok(s) => Ok(s),
        Err(e) => return Err(e),
    }?;
    match lp.as_str() {
        "AUTO" => Ok(LogPath::Auto),
        "NONE" => Ok(LogPath::Non),
        s => Ok(LogPath::Path(s.to_owned())),
    }
}

fn get_hash_str_field(prog: &Yaml, field: &str, default: HashMap<String, String>) -> Result<HashMap<String, String>, ConfigError> {
    let f = match &prog[field] {
        Yaml::BadValue => return Ok(default),
        Yaml::Hash(h) => Ok(h.clone()),
        _ => return Err(ConfigError::from_not_hash(field)),
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
                    return Err(ConfigError::from_hash_value_not_string(field))
                }
            };
            Ok((new_k, new_v))
        })
    .collect()
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

#[derive(Debug, Clone, PartialEq)]
pub enum LogPath {
    Path(String),
    Auto,
    Non,
}

impl LogPath {
    #[must_use]
    pub fn is_path(&self, path: &str) -> bool {
        match self {
            LogPath::Path(p) => p == path,
            LogPath::Auto => false,
            LogPath::Non => false,
        }
    }
}

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

    fn from_unreadable_file(e: std::io::Error) -> ConfigError {
        ConfigError::new(&format!("failed to read config file: {}", e))
    }

    fn from_invalid_cfg_file(e: ConfigError) -> ConfigError {
        ConfigError::new(&format!("invalid config file: {}", e))
    }

    fn from_invalid_value(field: &str) -> ConfigError {
        ConfigError::new(&format!("invalid value for field: {}", field))
    }

    fn from_not_string(field: &str) -> ConfigError {
        ConfigError::new(&format!("field `{}` should be a string", field))
    }

    fn from_not_number(field: &str) -> ConfigError {
        ConfigError::new(&format!("field `{}` should be a number", field))
    }

    fn from_not_bool(field: &str) -> ConfigError {
        ConfigError::new(&format!("field `{}` should be a number", field))
    }

    fn from_not_array(field: &str) -> ConfigError {
        ConfigError::new(&format!("field `{}` should be an array", field))
    }

    fn from_not_hash(field: &str) -> ConfigError {
        ConfigError::new(&format!("field `{}` should be a hashmap", field))
    }

    fn from_array_value_not_nbr(field: &str) -> ConfigError {
        ConfigError::new(&format!("array `{}` values should be numbers", field))
    }

    fn from_hash_value_not_string(field: &str) -> ConfigError {
        ConfigError::new(&format!("hashmap `{}` values should be strings", field))
    }

    fn from_invalid_yaml(e: yaml_rust::ScanError) -> ConfigError {
        ConfigError::new(&format!( "error scanning config file: {}", e.to_string()))
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

#[cfg(test)]
mod tests {
    use crate::cfg::{self, Config, RestartPolicy};
    use std::collections::HashMap;

    #[test]
    fn without_default_values() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    numprocs: 2
    umask: 777
    workingdir: \"/tmp\"
    autostart: false
    autorestart: never
    exitcodes:
      - 5
      - 2
      - 3
    startretries: 7
    starttime: 15
    stopsignal: KILL
    stoptime: 18
    stdout: \"/tmp/default.stdout\"
    stderr: \"/tmp/default.stderr\"
    env:
      STARTED_BY: taskmaster
      ANSWER: \"42\"";
        let c = Config::from_str(yaml).unwrap();
        assert_eq!(c.programs["cat0"].cmd, "/bin/cat");
        assert_eq!(c.programs["cat0"].numprocs, 2);
        assert_eq!(c.programs["cat0"].umask, 0o777);
        assert!(c.programs["cat0"].workingdir == Some("/tmp".to_string()));
        assert_eq!(c.programs["cat0"].autostart, false);
        assert_eq!(c.programs["cat0"].autorestart, RestartPolicy::Never);
        assert_eq!(c.programs["cat0"].exitcodes, vec![5, 2, 3]);
        assert_eq!(c.programs["cat0"].startretries, 7);
        assert_eq!(c.programs["cat0"].starttime, 15);
        assert_eq!(c.programs["cat0"].stopsignal.as_str(), "SIGKILL");
        assert_eq!(c.programs["cat0"].stoptime, 18);
        assert!(c.programs["cat0"].stdout.is_path("/tmp/default.stdout"));
        assert!(c.programs["cat0"].stderr.is_path("/tmp/default.stderr"));
        assert_eq!(c.programs["cat0"].env,
            HashMap::from([("STARTED_BY".to_string(), "taskmaster".to_string()),
                ("ANSWER".to_string(), "42".to_string())
            ])
        )
    }

    #[test]
    fn with_default_values_only() {
        let yaml = "
programs:
    cat:
      cmd: \"/bin/cat\"";
        let c = Config::from_str(yaml).unwrap();
        assert_eq!(c.programs["cat"].cmd, "/bin/cat");
        assert_eq!(c.programs["cat"].numprocs, cfg::DFLT_NUMPROCS);
        assert_eq!(c.programs["cat"].umask, cfg::DFLT_UMASK);
        assert!(c.programs["cat"].workingdir ==  cfg::DFLT_CWD);
        assert_eq!(c.programs["cat"].autostart, cfg::DFLT_AUTOSTART);
        assert_eq!(c.programs["cat"].autorestart, cfg::DFLT_AUTORESTART);
        assert_eq!(c.programs["cat"].exitcodes, cfg::DFLT_EXITCODES);
        assert_eq!(c.programs["cat"].startretries, cfg::DFLT_STARTRETRIES);
        assert_eq!(c.programs["cat"].starttime, cfg::DFLT_STARTTIME);
        assert_eq!(c.programs["cat"].stopsignal.as_str(), "SIGTERM");
        assert_eq!(c.programs["cat"].stoptime, cfg::DFLT_STOPTIME);
        assert_eq!(c.programs["cat"].stdout, cfg::LogPath::Auto);
        assert_eq!(c.programs["cat"].stderr, cfg::LogPath::Auto);
        assert_eq!(c.programs["cat"].env, HashMap::new());
    }

    #[test]
    fn with_invalid_numprocs() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    numprocs: error";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_umask() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    umask: error";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_workingdir() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    workingdir: 10";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_autostart() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    autostart: 10";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_autorestart() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    autorestart: 10";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_exitcodes() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    exitcodes:
      - err1
      - err2";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_startretries() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    startretries: err";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_starttime() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    starttime: err";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_stopsignal() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    stopsignal: err";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_stoptime() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    stoptime: err";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_stdout() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    stdout: 10";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_stderr() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    stderr: 10";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }

    #[test]
    fn with_invalid_env() {
        let yaml = "
programs:
  cat:
    cmd: \"/bin/cat\"
    stderr: 10";
        let c = Config::from_str(yaml);
        assert!(c.is_err())
    }
}
