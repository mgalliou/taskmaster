use std::fs;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
pub struct ProgramConfig {
    pub cmd: String,
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
pub struct Config {
    pub programs: HashMap<String, ProgramConfig>
}

fn get_field(prog: (&Yaml, &Yaml), field: &str) -> String {
    match prog.1[field].as_str() {
        Some(s) => s.to_string(),
        None => panic!("field `{}` no found", field)
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
            cmd: get_field(e, "cmd"),
            stdout: get_field(e, "stdout"),
            stderr: get_field(e, "stderr"),
        });
    }
    return prog_conf
}

pub fn get(path: String) -> Config {
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
    use super::get;

    #[test]
    fn it_works() {
        let c = get("cfg/good/cat.yaml".to_string());
        assert_eq!(c.programs["cat"].cmd, "/bin/cat");
        assert_eq!(c.programs["cat"].stdout, "/tmp/cat.stdout");
        assert_eq!(c.programs["cat"].stderr, "/tmp/cat.stderr");
    }
}
