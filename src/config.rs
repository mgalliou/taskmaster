use std::fs;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub enum RestartPolicy {
    Always,
    Never,
    Unexpected
}

impl RestartPolicy {
    fn from_str(s: &str) -> RestartPolicy {
        match s {
            "always" => RestartPolicy::Always,
            "never" => RestartPolicy::Never,
            "unexpected" => RestartPolicy::Unexpected,
            &_ => panic!("RestartPolicy is not one of `always`, `never` or `unexpected`: {}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramConfig {
    pub cmd: String,
    pub numprocs: i64,
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

#[derive(Debug)]
pub struct Config {
    pub programs: HashMap<String, ProgramConfig>
}

fn get_enum_field(prog: (&Yaml, &Yaml), field: &str) -> RestartPolicy {
    match prog.1[field].as_str() {
        Some(s) => RestartPolicy::from_str(s),
        None => panic!("field not found or invalid: {}", field)
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
    f.into_iter().map(|n| n.as_i64().expect("field not a number")).collect::<Vec<i64>>()
}

fn get_num_field(prog: (&Yaml, &Yaml), field: &str) -> i64 {
    match prog.1[field].as_i64() {
        Some(s) => s,
        None => panic!("field not found or not a number: {}", field)
    }
}

fn get_hash_str_field(prog: (&Yaml, &Yaml), field: &str) -> HashMap<String, String> {
    let f = match prog.1[field].clone().into_hash() {
        Some(h) => h,
        None => panic!("invalid or field not found: {}", field),
    };
    f.into_iter().map(|n| -> (String, String) {
        (n.0.into_string().expect("hashmap key should be a string"),
        n.1.into_string().expect("hashmap value should be a string"))
    }).collect::<HashMap<String, String>>()
}

fn get_str_field(prog: (&Yaml, &Yaml), field: &str) -> String {
    match prog.1[field].as_str() {
        Some(s) => s.to_string(),
        None => panic!("field not found or not a string: {}", field)
    }
}

fn get_name(prog: (&Yaml, &Yaml)) -> String {
    match prog.0.as_str() {
        Some(s) => s.to_string(),
        None => panic!("field is not as string"),
    }
}

fn get_prog_conf(yaml: &Yaml) -> HashMap<String, ProgramConfig> {
    let mut prog_conf: HashMap<String, ProgramConfig> = HashMap::new();
    let programs = yaml["programs"].as_hash().expect("no program field found");

    for e in programs.into_iter() {
        prog_conf.insert(get_name(e), ProgramConfig {
            cmd: get_str_field(e, "cmd"),
            numprocs: get_num_field(e, "numprocs"),
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
    return prog_conf
}

pub fn from_file(path: String) -> Config {
    let yaml_str = match fs::read_to_string(path) {
        Ok(f) => f,
        Err(error) => panic!("Problem reading the file: {:?}", error),
    };
    let yaml = match YamlLoader::load_from_str(&yaml_str) {
        Ok(yaml) => yaml,
        Err(error) => panic!("Problem converting string to yaml object: {:?}", error),
    };
    return Config { programs: get_prog_conf(&yaml[0]) };
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::config;

    #[test]
    fn it_works() {
        let c = config::from_file("cfg/good/cat.yaml".to_string());
        assert_eq!(c.programs["cat"].cmd, "/bin/cat");
        assert_eq!(c.programs["cat"].numprocs, 1);
        assert_eq!(c.programs["cat"].autostart, true);
        assert_eq!(c.programs["cat"].autorestart, config::RestartPolicy::Unexpected);
        assert_eq!(c.programs["cat"].exitcodes, vec![0, 2]);
        assert_eq!(c.programs["cat"].startretries, 3);
        assert_eq!(c.programs["cat"].starttime, 5);
        assert_eq!(c.programs["cat"].stopsignal, "TERM");
        assert_eq!(c.programs["cat"].stoptime, 10);
        assert_eq!(c.programs["cat"].stdout, "/tmp/cat.stdout");
        assert_eq!(c.programs["cat"].stderr, "/tmp/cat.stderr");
        assert_eq!(c.programs["cat"].env, HashMap::from([("STARTED_BY".to_string(), "taskmaster".to_string()),("ANSWER".to_string(), "42".to_string())]));
    }
}
