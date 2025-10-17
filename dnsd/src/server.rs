use std::{
    net::{Ipv4Addr, UdpSocket},
    time::Duration,
};

use crate::{
    SerDe,
    dns::{Message, ResourceRecord, ResourceRecordClass, ResourceRecordType},
};

fn process(request: Message) -> std::io::Result<Message> {
    tracing::debug!("request: {request:x?}");
    let mut header = request.header;
    header.set_response();
    header.set_aa();
    header.set_ra();
    header.answer_resource_record_count = 1;
    let questions = request.questions;
    let name = questions[0].name.clone();
    let addr = Ipv4Addr::new(10, 11, 12, 13);
    let resource_record = ResourceRecord::new(
        name,
        ResourceRecordType::A,
        ResourceRecordClass::IN,
        Duration::from_secs(127),
        addr.to_bits().to_be_bytes().to_vec(),
    );
    let additional_resource_records = request.additional_resource_records;
    let response = Message::new(
        header,
        questions,
        vec![resource_record],
        vec![],
        additional_resource_records,
    );
    tracing::debug!("response: {response:x?}");
    Ok(response)
}

pub fn serve() -> std::io::Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8000")?;
    let mut buf = [0; 4096];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, remote_addr)) => {
                println!("rcvd {n} from {remote_addr:?}");
                let raw_request = &buf[..n];
                println!("raw: [{raw_request:?}]");
                let request = Message::deserialize(raw_request)?;
                let response = process(request)?;
                let mut v = Vec::new();
                response.serialize(&mut v)?;
                socket.send_to(&v, remote_addr).unwrap();
            }
            Err(e) => println!("error: {e:?}"),
        }
    }
    Ok(())
}
