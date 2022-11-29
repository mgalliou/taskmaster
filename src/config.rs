use core::fmt;
use std::collections::HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::process::Stdio;
use std::str::FromStr;
use nix::sys::signal::Signal;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct ConfigError {
    details: String
}

impl ConfigError {
    fn new(msg: &str) -> ConfigError {
        ConfigError{details: msg.to_string()}
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
    fn from_yaml(name: &String, conf: &Yaml) -> Result<ProgramConfig, ConfigError> {
        let c = ProgramConfig {
            name: name.clone(),
            cmd: get_str_field(conf, "cmd")?,
            numprocs: get_num_field(conf, "numprocs")?,
            umask: get_umask_field(conf, "umask")?,
            workingdir: get_str_field(conf, "workingdir")?,
            autostart: get_bool_field(conf, "autostart")?,
            autorestart: get_enum_field(conf, "autorestart")?,
            exitcodes: get_num_vec_field(conf, "exitcodes")?,
            startretries: get_num_field(conf, "startretries")?,
            starttime: get_num_field(conf, "starttime")?,
            stopsignal: get_stop_signal(conf)?,
            stoptime: get_num_field(conf, "stoptime")?,
            stdout: get_str_field(conf, "stdout")?,
            stderr: get_str_field(conf, "stderr")?,
            env: get_hash_str_field(conf, "env")?,
        };
        Ok(c)
    }

    pub fn open_stdout(&self) -> Stdio {
        if self.stdout.is_empty() {
            Stdio::null()
        } else {
            match File::create(&self.stdout) {
                Ok(f) => Stdio::from(f),
                //TODO: log file opening failure
                Err(_) => Stdio::null()
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
                Err(_) => Stdio::null()
            }
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub programs: HashMap<String, ProgramConfig>,
}

fn get_enum_field(prog: &Yaml, field: &str) -> Result<RestartPolicy, ConfigError> {
    match prog[field].as_str() {
        Some(s) => match RestartPolicy::from_str(s) {
            Ok(rp) => Ok(rp),
            Err(_) => Err(ConfigError::new(&format!("invalid value for field: {}", field))),
        },
        None => Ok(RestartPolicy::Always),
    }
}

fn get_bool_field(prog: &Yaml, field: &str) -> Result<bool, ConfigError> {
    match prog[field].as_bool() {
        Some(s) => Ok(s),
        None => Err(ConfigError::new(&format!("field not found or not a boolean: {}", field)))
    }
}

fn get_umask_field(prog: &Yaml, field: &str) -> Result<u32, ConfigError> {
    match prog[field].as_i64() {
        Some(s) => Ok(u32::from_str_radix(s.to_string().as_str(), 8).unwrap()),
        None => Err(ConfigError::new(&format!("field not found or not a number: {}", field)))
    }
}

fn get_num_field(prog: &Yaml, field: &str) -> Result<i64, ConfigError> {
    match prog[field].as_i64() {
        Some(f) => Ok(f),
        None => Err(ConfigError::new(&format!("field not found or not a number: {}", field)))
    }
}

fn get_num_vec_field(prog: &Yaml, field: &str) -> Result<Vec<i64>, ConfigError> {
    let f = match prog[field].clone().into_vec() {
        Some(v) => Ok(v),
        None => Err(ConfigError::new(&format!("field not found: {}", field)))
    }?;
    f.into_iter().map(|n| match n.as_i64() {
        Some(n) => Ok(n),
        None => Err(ConfigError::new(&format!("invalid value for field: {}", field)))
    }).collect()
}

fn get_hash_str_field(prog: &Yaml, field: &str) -> Result<HashMap<String, String>, ConfigError> {
    let f = match prog[field].clone().into_hash() {
        Some(h) => Ok(h),
        None => Err(ConfigError::new(&format!("field not found or invalid: {}", field)))
    }?;
    f.into_iter()
        .map(|(k, v)| {
            let new_k = match k.into_string() {
                Some(k) => k,
                None => return Err(ConfigError::new(&format!("invalid key in field: {}", field)))
            };
            let new_v = match v.into_string() {
                Some(v) => v,
                None => return Err(ConfigError::new(&format!("invalid key in field: {}", field)))
            };
            Ok((new_k, new_v))
        })
    .collect()
}

fn get_str_field(prog: &Yaml, field: &str) -> Result<String, ConfigError> {
    match prog[field].as_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(ConfigError::new(&format!("field not found or not a string: {}", field))),
    }
}

fn gen_name(numprocs: i64, base_name: &String, i: i64) -> String {
    if numprocs == 1 {
        base_name.clone()
    } else {
        base_name.clone() + &i.to_string()
    }
}

fn get_stop_signal(prog: &Yaml) -> Result<Signal, ConfigError> {
    let ss = get_str_field(prog, "stopsignal")?;
    match ("SIG".to_owned() + &ss).parse::<Signal>() {
        Ok(s) => Ok(s),
        Err(_) => Ok(Signal::SIGTERM),
    }
}

fn get_prog_conf(yaml: &Yaml) -> Result<Config, ConfigError> {
    let mut programs: HashMap<String, ProgramConfig> = HashMap::new();
    let progs_yaml = yaml["programs"].as_hash().expect("no program field found");

    for (name, conf)  in progs_yaml.into_iter() {
        let base_name = name.as_str().unwrap();
        let numprocs = get_num_field(conf, "numprocs")?;
        for i in 0..numprocs {
            let name = gen_name(numprocs, &base_name.to_string(), i);
            let pc = ProgramConfig::from_yaml(&name, conf);
            programs.insert(name, pc.unwrap());
        }
    }
    Ok(Config { programs })
}


pub fn from_str(str: String) -> Result<Config, ConfigError> {
    match YamlLoader::load_from_str(&str) {
        Ok(yaml) => get_prog_conf(&yaml[0]),
        Err(e) => Err(ConfigError::new(&format!("error scanning config file: {}", e.to_string()))),
    }
}

pub fn from_file(path: String) -> Result<Config, ConfigError> {
    //TODO:should return Result
    let yaml_str = match fs::read_to_string(path) {
        Ok(f) => Ok(f),
        Err(e) => Err(ConfigError::new(&format!("Failed to read config file: {}", e)))
    }?;
    match from_str(yaml_str) {
        Ok(c) => Ok(c),
        Err(e) => Err(ConfigError::new(&format!("Failed to parse yaml: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use crate::config;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let c = config::from_file("cfg/good/cat.yaml".to_string()).unwrap();
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
