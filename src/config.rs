use std::collections::HashMap;
use std::fs::{self, File};
use std::process::Stdio;
use yaml_rust::{Yaml, YamlLoader, ScanError};

#[derive(Debug, Clone, PartialEq)]
pub enum RestartPolicy {
    Always,
    Never,
    Unexpected,
}

impl RestartPolicy {
    fn from_str(s: &str) -> RestartPolicy {
        match s {
            "always" => RestartPolicy::Always,
            "never" => RestartPolicy::Never,
            "unexpected" => RestartPolicy::Unexpected,
            &_ => panic!(
                "RestartPolicy is not one of `always`, `never` or `unexpected`: {}",
                s
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramConfig {
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
    pub stopsignal: String,
    pub stoptime: i64,
    pub env: HashMap<String, String>,
}

impl ProgramConfig {
    pub fn open_stdout(&self) -> Stdio {
        if self.stdout.is_empty() {
            Stdio::null()
        } else {
            //TODO: handle error correctly
            Stdio::from(File::create(&self.stdout).unwrap())
        }
    }

    pub fn open_stderr(&self) -> Stdio {
        if self.stderr.is_empty() {
            Stdio::null()
        } else {
            //TODO: handle error correctly
            Stdio::from(File::create(&self.stderr).unwrap())
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub programs: HashMap<String, ProgramConfig>,
}

fn get_enum_field(prog: (&Yaml, &Yaml), field: &str) -> RestartPolicy {
    match prog.1[field].as_str() {
        Some(s) => RestartPolicy::from_str(s),
        None => panic!("field not found or invalid: {}", field),
    }
}

fn get_bool_field(prog: (&Yaml, &Yaml), field: &str) -> bool {
    match prog.1[field].as_bool() {
        Some(s) => s,
        None => panic!("field not found or not a boolean: {}", field),
    }
}

fn get_num_vec_field(prog: (&Yaml, &Yaml), field: &str) -> Vec<i64> {
    let f = match prog.1[field].clone().into_vec() {
        Some(v) => v,
        None => panic!("field not found: {}", field),
    };
    //TODO: better error handling if a field is not a number
    f.into_iter()
        .map(|n| n.as_i64().expect("field not a number"))
        .collect::<Vec<i64>>()
}

fn get_umask_field(prog: (&Yaml, &Yaml), field: &str) -> u32 {
    match prog.1[field].as_i64() {
        Some(s) => u32::from_str_radix(s.to_string().as_str(), 8).unwrap(),
        None => panic!("field not found or not a number: {}", field),
    }
}

fn get_num_field(prog: (&Yaml, &Yaml), field: &str) -> i64 {
    match prog.1[field].as_i64() {
        Some(s) => s,
        None => panic!("field not found or not a number: {}", field),
    }
}

fn get_hash_str_field(prog: (&Yaml, &Yaml), field: &str) -> HashMap<String, String> {
    let f = match prog.1[field].clone().into_hash() {
        Some(h) => h,
        None => panic!("invalid or field not found: {}", field),
    };
    f.into_iter()
        .map(|n| -> (String, String) {
            (
                n.0.into_string().expect("hashmap key should be a string"),
                n.1.into_string().expect("hashmap value should be a string"),
            )
        })
        .collect::<HashMap<String, String>>()
}

fn get_str_field(prog: (&Yaml, &Yaml), field: &str) -> String {
    match prog.1[field].as_str() {
        Some(s) => s.to_string(),
        None => panic!("field not found or not a string: {}", field),
    }
}

fn get_name(prog: (&Yaml, &Yaml)) -> String {
    match prog.0.as_str() {
        Some(s) => s.to_string(),
        None => panic!("field is not as string"),
    }
}

fn gen_name(numprocs: i64, base_name: &String, i: i64) -> String {
    if numprocs == 1 {
        base_name.clone()
    } else {
        base_name.clone() + &i.to_string()
    }
}

fn get_prog_conf(yaml: &Yaml) -> Config {
    let mut programs: HashMap<String, ProgramConfig> = HashMap::new();
    let progs_yaml = yaml["programs"].as_hash().expect("no program field found");

    for e in progs_yaml.into_iter() {
        let base_name = get_name(e);
        let numprocs = get_num_field(e, "numprocs");
        for i in 0..numprocs {
            programs.insert(
                gen_name(numprocs, &base_name, i),
                ProgramConfig {
                    cmd: get_str_field(e, "cmd"),
                    numprocs: get_num_field(e, "numprocs"),
                    umask: get_umask_field(e, "umask"),
                    workingdir: get_str_field(e, "workingdir"),
                    autostart: get_bool_field(e, "autostart"),
                    autorestart: get_enum_field(e, "autorestart"),
                    exitcodes: get_num_vec_field(e, "exitcodes"),
                    startretries: get_num_field(e, "startretries"),
                    starttime: get_num_field(e, "starttime"),
                    stopsignal: get_str_field(e, "stopsignal"),
                    stoptime: get_num_field(e, "stoptime"),
                    stdout: get_str_field(e, "stdout"),
                    stderr: get_str_field(e, "stderr"),
                    env: get_hash_str_field(e, "env"),
                });
        }
    }
    Config { 
        programs,
    }
}

pub fn from_str(str: String) -> Result<Config, ScanError> {
    match YamlLoader::load_from_str(&str) {
        Ok(yaml) => Ok(get_prog_conf(&yaml[0])),
        Err(e) => Err(e),
    }
}

pub fn from_file(path: String) -> Config {
    //TODO:should return Result
    let yaml_str = match fs::read_to_string(path) {
        Ok(f) => f,
        Err(e) => panic!("Failed to read config file: {:?}", e),
    };
    match from_str(yaml_str) {
        Ok(c) => c,
        Err(e) => panic!("Failed to parse yaml: {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use crate::config;
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        let c = config::from_file("cfg/good/cat.yaml".to_string());
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
        assert_eq!(c.programs["cat"].stopsignal, "TERM");
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
