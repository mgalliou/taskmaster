use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use taskmaster::launch_process;
use taskmaster::config::Config;
use taskmaster::config;

fn get_command(line: String, conf: &Config) -> () {
    let command = line.split_whitespace().collect::<Vec<&str>>();
    match command[0] {
        "start" => launch_process::start(command, conf),
        "status" => println!("status"),
        "stop" => println!("stop"),
        "restart" => println!("restart"),
        "reload" => println!("reload"),
        "exit" => println!("exit"),
        &_ => return
    }
}

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let conf = config::from_file("cfg/good/cat.yaml".to_string());
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("taskmaster>");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                get_command(line.to_string(), &conf);
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

