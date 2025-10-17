use std::io::{Read, Write};

use crate::{
    SerDe,
    dns::{ResourceRecordClass, ResourceRecordName, ResourceRecordType},
    read_u16_be, write_u16_be,
};

#[derive(Debug)]
pub struct Question {
    pub name: ResourceRecordName,
    r#type: ResourceRecordType,
    class: ResourceRecordClass,
}

impl SerDe for Question {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        self.name.serialize(&mut w)?;
        write_u16_be(&mut w, self.r#type as u16)?;
        write_u16_be(&mut w, self.class as u16)?;
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let name = ResourceRecordName::deserialize(&mut r)?;
        let r#type =
            unsafe { std::mem::transmute::<u16, ResourceRecordType>(read_u16_be(&mut r)?) };
        let class =
            unsafe { std::mem::transmute::<u16, ResourceRecordClass>(read_u16_be(&mut r)?) };
        Ok(Self {
            name,
            r#type,
            class,
        })
    }
}
