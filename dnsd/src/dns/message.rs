use std::io::{Read, Write};

use crate::{
    SerDe,
    dns::{header::Header, question::Question, record::ResourceRecord},
};

#[derive(Debug)]
pub(crate) struct Message {
    header: Header,
    questions: Vec<Question>,
    answer_resource_records: Vec<ResourceRecord>,
    authority_resource_records: Vec<u8>,
    additional_resource_records: Vec<u8>,
}

impl SerDe for Message {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        self.header.serialize(&mut w)?;
        for q in &self.questions {
            q.serialize(&mut w)?;
        }
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = Header::deserialize(&mut r)?;
        let question = Question::deserialize(&mut r)?;
        Ok(Self {
            header,
            questions: vec![question],
            answer_resource_records: vec![],
            authority_resource_records: vec![],
            additional_resource_records: vec![],
        })
    }
}
