use std::fs;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

struct ProgramConfig {
    cmd: String,
    stdout: String,
    stderr: String,
}

pub struct Config {
    programs: HashMap<String, ProgramConfig>
}

fn get_prog_conf(yaml: Vec<Yaml>) -> HashMap<String, ProgramConfig> {
    let prog_conf: HashMap<String, ProgramConfig> = HashMap::new();
    let programs: HashMap<Yaml, Yaml>;

    for node in yaml {
        match node.into_hash() {
            Some(n) => match n.remove(&Yaml::String("programs".to_string())) {
                Some (p) => p.into_hash(),
                None => panic!("no 'programs' key in yaml"),
            },
            None => panic!("configuration file is empty"),
        }
    }
    prog_conf
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
    let conf: Config = Config { programs: get_prog_conf(yaml) };
    conf
}
