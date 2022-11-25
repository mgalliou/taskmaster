use std::io::{Result, Write, Read};
use std::os::unix::net::{UnixStream, UnixListener};

//TODO: check stream result
pub fn send_message(line: String) -> Result<()> {
    let mut stream = UnixStream::connect("taskmaster.socket")?;
    let mut response = String::new();
    stream.write(line.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    stream.read_to_string(&mut response).expect("failed to read stream");
    println!("ctl response: {}", response);
    Ok(())
}

//TODO: check stream result
pub fn read_message(listener: &UnixListener) -> String {
    let mut response = String::new();
    let (mut stream, _socket) = listener.accept().expect("fail accept");
    stream.read_to_string(&mut response).expect("failed to read stream");
    response
}

