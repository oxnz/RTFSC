use std::io::{Read, Write};

use crate::{
    SerDe,
    dns::{Header, Question, ResourceRecord},
};

/**
 * The answer, authority, and additional sections all share the same format:
 *  a variable number of resource records, where the number of records is specified in the corresponding count field in the header
 */
#[derive(Debug)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answer_resource_records: Vec<ResourceRecord>,
    pub authority_resource_records: Vec<ResourceRecord>,
    pub additional_resource_records: Vec<ResourceRecord>,
}

impl Message {
    pub fn new(
        header: Header,
        questions: Vec<Question>,
        answer_resource_records: Vec<ResourceRecord>,
        authority_resource_records: Vec<ResourceRecord>,
        additional_resource_records: Vec<ResourceRecord>,
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
        for item in &self.authority_resource_records {
            item.serialize(&mut w)?;
        }
        for item in &self.additional_resource_records {
            item.serialize(&mut w)?;
        }
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let header = Header::deserialize(&mut r)?;
        let mut questions = Vec::new();
        for _i in 0..header.question_resource_record_count {
            let question = Question::deserialize(&mut r)?;
            questions.push(question);
        }
        let mut answer_resource_records = Vec::new();
        for _i in 0..header.answer_resource_record_count {
            let record = ResourceRecord::deserialize(&mut r)?;
            answer_resource_records.push(record);
        }
        let mut authority_resource_records = Vec::new();
        for _i in 0..header.authority_resource_record_count {
            let record = ResourceRecord::deserialize(&mut r)?;
            authority_resource_records.push(record);
        }
        let mut additional_resource_records = Vec::new();
        for _i in 0..header.additional_resource_record_count {
            let record = ResourceRecord::deserialize(&mut r)?;
            additional_resource_records.push(record);
        }
        Ok(Self {
            header,
            questions,
            answer_resource_records,
            authority_resource_records,
            additional_resource_records,
        })
    }
}
