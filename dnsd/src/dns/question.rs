use std::io::{Read, Write};

use crate::{SerDe, dns::record::ResourceRecordName, read_u16};

#[derive(Debug)]
pub(crate) struct Question {
    name: ResourceRecordName,
    r#type: u16,
    class: u16,
}

impl SerDe for Question {
    fn serialize<W: Write>(&self, w: W) -> std::io::Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let name = ResourceRecordName::deserialize(&mut r)?;
        let r#type = read_u16(&mut r)?;
        let class = read_u16(&mut r)?;
        Ok(Self {
            name,
            r#type,
            class,
        })
    }
}
