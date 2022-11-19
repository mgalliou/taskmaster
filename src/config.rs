use std::fs;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

#[derive(Debug)]
struct ProgramConfig {
    cmd: String,
    stdout: String,
    stderr: String,
}

#[derive(Debug)]
pub struct Config {
    programs: HashMap<String, ProgramConfig>
}

fn get_prog_conf(yaml: &Yaml) -> HashMap<String, ProgramConfig> {
    let mut prog_conf: HashMap<String, ProgramConfig> = HashMap::new();
    let programs = &yaml["programs"];

    for prog in programs.as_hash().into_iter() {
        for e in prog.into_iter() {
            let name = match e.0.as_str() {
                Some(s) => s.to_string(),
                None => panic!("field is not as string"),
            };
            let cmd = &e.1["cmd"].as_str().unwrap().to_string();
            let stdout = &e.1["stdout"].as_str().unwrap().to_string();
            let stderr = &e.1["stderr"].as_str().unwrap().to_string();
            prog_conf.insert(name, ProgramConfig { cmd: cmd.to_string(), stdout: stdout.to_string(), stderr: stderr.to_string() });
        }
    }
    /*
    for node in yaml {
        match node.into_hash() {
            Some(n) => match n.remove(&Yaml::String("programs".to_string())) {
                Some (p) => p.into_hash(),
                None => panic!("no 'programs' key in yaml"),
            },
            None => panic!("configuration file is empty"),
        }
    }
    */
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
    return Config { programs: get_prog_conf(&yaml[0]) };
}
