use std::io::{Write, Read};
use std::os::unix::net::UnixStream;

use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use taskmaster::config::{self, Config};
use taskmaster::daemon::start::{self};
use taskmaster::daemon::ProcessInfo;

fn send_message(line: String) -> std::io::Result<()> {
    let mut stream = UnixStream::connect("/tmp/taskmaster.socket")?;
    stream.write_all(line.as_bytes())?;
    let mut response = String::new();
    Ok(())
}

fn main() -> Result<()> {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("taskmaster>");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                send_message(line);
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
