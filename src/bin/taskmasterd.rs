extern crate yaml_rust;

use taskmaster::config;

fn main() {
    println!("Running taskmaster daemon");
    let conf = config::from_file("taskmaster.yaml".to_string());
    println!("{:?}", conf);
}
