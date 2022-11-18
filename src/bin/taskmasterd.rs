extern crate yaml_rust;

use taskmaster::config;

fn main() {
    println!("Running taskmaster daemon");
    config::get("taskmaster.yaml".to_string());
}
