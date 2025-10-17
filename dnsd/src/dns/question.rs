use std::io::{Read, Write};

use crate::{SerDe, dns::record::ResourceRecordName, read_u16_be, write_u16_be};

#[repr(u16)]
#[derive(Debug, Clone, Copy)]
enum ResourceRecordType {
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

#[derive(Debug)]
pub(crate) struct Question {
    name: ResourceRecordName,
    r#type: ResourceRecordType,
    class: u16,
}

impl SerDe for Question {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        self.name.serialize(&mut w)?;
        write_u16_be(&mut w, self.r#type as u16)?;
        write_u16_be(&mut w, self.class)?;
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let name = ResourceRecordName::deserialize(&mut r)?;
        let r#type =
            unsafe { std::mem::transmute::<u16, ResourceRecordType>(read_u16_be(&mut r)?) };
        let class = read_u16_be(&mut r)?;
        Ok(Self {
            name,
            r#type,
            class,
        })
    }
}
