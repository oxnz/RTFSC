use std::io::{Read, Write};

use crate::{SerDe, dns::record::ResourceRecordName, read_u16_be, write_u16_be};

#[derive(Debug)]
pub(crate) struct Question {
    name: ResourceRecordName,
    r#type: u16,
    class: u16,
}

impl SerDe for Question {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        self.name.serialize(&mut w)?;
        write_u16_be(&mut w, self.r#type)?;
        write_u16_be(&mut w, self.class)?;
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let name = ResourceRecordName::deserialize(&mut r)?;
        let r#type = read_u16_be(&mut r)?;
        let class = read_u16_be(&mut r)?;
        Ok(Self {
            name,
            r#type,
            class,
        })
    }
}
