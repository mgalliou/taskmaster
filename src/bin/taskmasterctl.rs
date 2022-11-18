use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

fn get_command(line: String) -> () {
    let mut command = line.split_whitespace();
    match command.next() {
        Some("status") => println!("status"),
        Some("start") => println!("start"),
        Some("stop") => println!("stop"),
        Some("restart") => println!("restart"),
        Some("reload") => println!("reload"),
        Some("exit") => println!("exit"),
        None | _ => println!("none")
    }
}

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                get_command(line.to_string());
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

