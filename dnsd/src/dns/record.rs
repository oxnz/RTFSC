use std::io::{Read, Write};

use crate::SerDe;

#[derive(Debug)]
pub(crate) struct ResourceRecord {
    name: ResourceRecordName,
    r#type: u16,
    class: u16,
    time_to_live: u32,
    data_len: u16,
    data: Vec<u8>,
}

impl SerDe for ResourceRecord {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        self.name.serialize(&mut w)?;
        todo!()
    }

    fn deserialize<R: Read>(r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug)]
pub(crate) struct ResourceRecordName {
    data: Vec<Vec<u8>>,
}

impl SerDe for ResourceRecordName {
    fn serialize<W: Write>(&self, w: W) -> std::io::Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut data = Vec::new();
        let mut len = vec![0];
        loop {
            r.read_exact(&mut len)?;
            let n = len[0] as usize;
            if n == 0 {
                break;
            }
            let mut buf = vec![0; n];
            r.read_exact(&mut buf)?;
            data.push(buf);
        }
        Ok(Self { data })
    }
}
