use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() {
    let addr = "127.0.0.1:8000";
    let tcp_listener = TcpListener::bind(addr).expect("bind");
    println!("listening on {addr}");
    for stream in tcp_listener.incoming() {
        match stream {
            Ok(stream) => handle_stream(stream),
            Err(e) => eprintln!("error: {e:?}"),
        }
    }
}

fn handle_stream(mut stream: TcpStream) {
    println!("accet: {:?}", stream);
    let mut buffer = [0; 10];
    loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n > 0 {
                    println!("rcvd {n}");
                    stream.write(&buffer).expect("send");
                } else {
                    stream.write(b"bye").expect("send");
                }
            }
            Err(e) => eprintln!("{e:?}"),
        }
    }
}
