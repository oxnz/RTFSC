use std::io::{Read, Write};

use crate::{
    SerDe,
    dns::{Header, Question, ResourceRecord},
};

#[derive(Debug)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answer_resource_records: Vec<ResourceRecord>,
    pub authority_resource_records: Vec<u8>,
    pub additional_resource_records: Vec<u8>,
}

impl Message {
    pub fn new(
        header: Header,
        questions: Vec<Question>,
        answer_resource_records: Vec<ResourceRecord>,
        authority_resource_records: Vec<u8>,
        additional_resource_records: Vec<u8>,
    ) -> Self {
        Self {
            header,
            questions,
            answer_resource_records,
            authority_resource_records,
            additional_resource_records,
        }
    }
}

impl SerDe for Message {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        self.header.serialize(&mut w)?;
        for item in &self.questions {
            item.serialize(&mut w)?;
        }
        for item in &self.answer_resource_records {
            item.serialize(&mut w)?;
        }
        w.write_all(&self.authority_resource_records)?;
        w.write_all(&self.additional_resource_records)?;
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = Header::deserialize(&mut r)?;
        let question = Question::deserialize(&mut r)?;
        let mut additional_resource_records = Vec::new();
        if header.additional_resource_record_count != 0 {
            r.read_to_end(&mut additional_resource_records)?;
        }
        Ok(Self {
            header,
            questions: vec![question],
            answer_resource_records: vec![],
            authority_resource_records: vec![],
            additional_resource_records,
        })
    }
}
