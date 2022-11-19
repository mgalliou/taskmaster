extern crate yaml_rust;

use taskmaster::config;

fn main() {
    println!("Running taskmaster daemon");
    let conf = config::get("taskmaster.yaml".to_string());
    println!("{:?}", conf);
}
