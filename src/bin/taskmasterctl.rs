use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use taskmaster::config::{self, Config};
use taskmaster::launch_process::{self, ProcessInfo};

fn get_command(line: String, conf: &Config, _proc_list: &ProcessInfo) -> ProcessInfo {
    let cmd: Vec<&str> = line.split_whitespace().collect::<Vec<&str>>();
    match cmd[0] {
        "start" => launch_process::start(cmd, conf),
        //"status" => launch_proces::status(command, conf),
        //"stop" => launch_proces::stop(command, conf),
        //"restart" => launch_proces::restart(command, conf),
        //"reload" => launch_proces::reload(command, conf),
        //"exit" => launch_proces::exit(command, conf),
        &_ => Vec::new(),
    }
}

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let conf = config::from_file("cfg/good/cat.yaml".to_string());
    let mut proc_list: launch_process::ProcessInfo = Vec::new();
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("taskmaster>");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                proc_list.append(&mut get_command(line.to_string(), &conf, &proc_list));
                println!(" {:?}", proc_list);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}
