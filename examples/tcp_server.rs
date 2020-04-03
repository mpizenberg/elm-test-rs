use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Run both following commands in two terminals
/// cargo run --example tpc_server
/// node examples/tcp_client.js
fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:30000")?;

    // Accept connections and process them serially
    for stream in listener.incoming() {
        let stream = stream.expect("woops stream");
        std::thread::spawn(move || {
            handle_client(stream);
        });
    }

    Ok(())
}

/// Hanle connection with client TCP socket
fn handle_client(mut stream: TcpStream) {
    println!("New connection: {}", stream.peer_addr().unwrap());
    let mut buf = "received: ".to_string();
    stream.set_nodelay(true).expect("woops nodelay");
    stream.write(b"Hello from server").expect("woops write");
    loop {
        match read_message(&mut stream, &mut buf) {
            // If the message was of length 0,
            // it means that the connection was closed.
            Ok(0) => break,
            Ok(_) => println!("{}", buf),
            Err(e) => panic!("encountered IO error: {}", e),
        };
    }
}

/// Read until a message is the string "EOF"
fn read_message(stream: &mut TcpStream, string: &mut String) -> std::io::Result<usize> {
    let mut buf_vec = vec![];
    let mut buf = [0; 1024];
    let mut nb_read = 1024;
    let mut eof = false;
    while nb_read == 1024 || eof == false {
        nb_read = stream.read(&mut buf[..])?;
        println!("nb_read: {}", nb_read);
        if nb_read == 0 || b"EOF" == &buf[..nb_read] {
            eof = true;
        } else {
            buf_vec.extend(&buf[..nb_read]);
        }
    }
    string.push_str(std::str::from_utf8(&buf_vec).expect("woops utf8"));
    println!("ended read_to_string");
    Ok(buf_vec.len())
}
