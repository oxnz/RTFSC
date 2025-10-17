use std::io::{Read, Write};

use crate::{SerDe, read_u16};

#[derive(Debug)]
pub(crate) struct Header {
    /** A 16-bit field identifying a specific DNS transaction.
     * The transaction ID is created by the message originator and
     * is copied by the responder into its response message.
     * Using the transaction ID, the DNS client can match responses to its requests.
     */
    transaction_id: u16,
    flags: u16,
    question_resource_record_count: u16,
    answer_resource_record_count: u16,
    authority_resource_record_count: u16,
    additional_resource_record_count: u16,
}

impl SerDe for Header {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(&self.transaction_id.to_be_bytes())?;
        w.write_all(&self.flags.to_be_bytes())?;
        w.write_all(&self.question_resource_record_count.to_be_bytes())?;
        w.write_all(&self.answer_resource_record_count.to_be_bytes())?;
        w.write_all(&self.authority_resource_record_count.to_be_bytes())?;
        w.write_all(&self.additional_resource_record_count.to_be_bytes())?;
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            transaction_id: read_u16(&mut r)?,
            flags: read_u16(&mut r)?,
            question_resource_record_count: read_u16(&mut r)?,
            answer_resource_record_count: read_u16(&mut r)?,
            authority_resource_record_count: read_u16(&mut r)?,
            additional_resource_record_count: read_u16(&mut r)?,
        })
    }
}
