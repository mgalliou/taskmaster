use std::process::Child;

use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use taskmaster::launch_process;
use taskmaster::config::{Config, ProgramConfig};
use taskmaster::config;

fn get_command(line: String, conf: &Config, prog_list: &Vec<(ProgramConfig, Vec<Child>)>) -> Vec<(ProgramConfig, Vec<Child>)> {
    let command = line.split_whitespace().collect::<Vec<&str>>();
    match command[0] {
        "start" => launch_process::start(command, conf),
        //"status" => launch_proces::status(command, conf),
        //"stop" => launch_proces::stop(command, conf),
        //"restart" => launch_proces::restart(command, conf),
        //"reload" => launch_proces::reload(command, conf),
        //"exit" => launch_proces::exit(command, conf),
        &_ => Vec::new()
    }
}

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let conf = config::from_file("cfg/good/cat.yaml".to_string());
    let mut prog_list: Vec<(ProgramConfig, Vec<Child>)> = Vec::new();
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("taskmaster>");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                prog_list.append(&mut get_command(line.to_string(), &conf, &prog_list));
                println!(" {:?}", prog_list);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt")
}

