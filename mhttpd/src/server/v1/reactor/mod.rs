use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    net::TcpStream,
};

use crate::http::{Request, Response};

#[derive(Debug)]
struct Transport {
    stream: BufReader<TcpStream>,
    codec: Codec,
}

impl TryFrom<TcpStream> for Transport {
    type Error = std::io::Error;

    fn try_from(value: TcpStream) -> Result<Self, Self::Error> {
        value.set_nonblocking(true)?;
        Ok(Self {
            stream: BufReader::new(value),
            codec: Codec::default(),
        })
    }
}

impl Transport {
    pub fn recv(&mut self) -> std::io::Result<Request> {
        self.codec.try_decode(&mut self.stream)
    }

    pub fn send(&mut self, response: Response) -> std::io::Result<()> {
        // let writer = BufWriter::new(self.stream.get_mut());
        self.codec.try_encode(response, self.stream.get_mut())
    }
}

#[derive(Debug)]
struct Connection {
    transport: Transport,
}

#[derive(Debug, Default)]
struct Codec {
    recv_buffer: Vec<u8>,
    send_buffer: Vec<u8>,
}

impl Codec {
    pub fn try_decode<R: BufRead>(&mut self, stream: &mut R) -> std::io::Result<Request> {
        todo!()
    }

    pub fn try_encode<W: Write>(
        &mut self,
        response: Response,
        stream: &mut W,
    ) -> std::io::Result<()> {
        todo!()
    }
}

impl Connection {
    pub fn read_request(&mut self) -> std::io::Result<Request> {
        self.transport.recv()
    }

    pub fn send_response(&mut self, response: Response) -> std::io::Result<()> {
        self.transport.send(response)
    }

    fn flush_write(&self) -> std::io::Result<()> {
        todo!()
    }
}

#[derive(Debug)]
struct Client {
    connection: Connection,
    router: Router,
}

impl Client {
    pub fn on_readable(&mut self) -> std::io::Result<()> {
        match self.connection.read_request() {
            Ok(request) => {
                // self.connection.unsub_readable()?;
                let response = self.router.handle_request(request)?;
                self.connection.send_response(response)?;
            }
            Err(_) => todo!(),
        }
        Ok(())
    }

    pub fn on_writable(&mut self) {
        match self.connection.flush_write() {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    }
}

#[derive(Debug)]
struct Router {}

impl Router {
    pub fn handle_request(&self, request: Request) -> std::io::Result<Response> {
        todo!()
    }
}
