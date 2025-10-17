use std::{
    io::{Read, Write},
    time::Duration,
};

use crate::{SerDe, write_u16_be, write_u32_be};

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceRecordType {
    A = 0x01,
    NS = 0x02,
    CNAME = 0x05,
    PTR = 0x0C,
    MX = 0x0F,
    SRV = 0x21,
    IXFR = 0xFB,
    AXFR = 0xFC,
    All = 0xFF,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum ResourceRecordClass {
    // the Internet class
    IN = 0x0001,
}

#[derive(Debug)]
pub(crate) struct ResourceRecord {
    name: ResourceRecordName,
    r#type: ResourceRecordType,
    class: ResourceRecordClass,
    time_to_live: Duration,
    data: Vec<u8>,
}

impl ResourceRecord {
    pub fn new(
        name: ResourceRecordName,
        r#type: ResourceRecordType,
        class: ResourceRecordClass,
        time_to_live: Duration,
        data: Vec<u8>,
    ) -> Self {
        Self {
            name,
            r#type,
            class,
            time_to_live,
            data,
        }
    }
}

impl SerDe for ResourceRecord {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        self.name.serialize(&mut w)?;
        write_u16_be(&mut w, self.r#type as u16)?;
        write_u16_be(&mut w, self.class as u16)?;
        write_u32_be(&mut w, self.time_to_live.as_secs() as u32)?;
        write_u16_be(&mut w, self.data.len() as u16)?;
        w.write_all(&self.data)?;
        Ok(())
    }

    fn deserialize<R: Read>(r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ResourceRecordName {
    data: Vec<Vec<u8>>,
}

impl SerDe for ResourceRecordName {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        for part in &self.data {
            let len = part.len() as u8;
            w.write_all(&len.to_le_bytes())?;
            w.write_all(&part)?;
        }
        w.write_all(&(0u8.to_le_bytes()))?;
        Ok(())
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
