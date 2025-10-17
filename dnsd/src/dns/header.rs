use std::io::{Read, Write};

use crate::{SerDe, read_u16_be, write_u16_be};

#[derive(Debug)]
pub(crate) struct Header {
    /** A 16-bit field identifying a specific DNS transaction.
     * The transaction ID is created by the message originator and
     * is copied by the responder into its response message.
     * Using the transaction ID, the DNS client can match responses to its requests.
     */
    transaction_id: u16,
    flags: u16,
    pub(crate) question_resource_record_count: u16,
    pub(crate) answer_resource_record_count: u16,
    pub(crate) authority_resource_record_count: u16,
    pub(crate) additional_resource_record_count: u16,
}

impl Header {
    pub fn set_request(&mut self) {
        self.flags &= 0xEFFF;
    }

    pub fn set_response(&mut self) {
        self.flags |= 0x8000;
    }

    pub fn set_aa(&mut self) {
        self.flags |= 0x0400;
    }

    pub fn set_ra(&mut self) {
        self.flags |= 0x0080;
    }
}

impl SerDe for Header {
    fn serialize<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        write_u16_be(&mut w, self.transaction_id)?;
        write_u16_be(&mut w, self.flags)?;
        write_u16_be(&mut w, self.question_resource_record_count)?;
        write_u16_be(&mut w, self.answer_resource_record_count)?;
        write_u16_be(&mut w, self.authority_resource_record_count)?;
        write_u16_be(&mut w, self.additional_resource_record_count)?;
        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            transaction_id: read_u16_be(&mut r)?,
            flags: read_u16_be(&mut r)?,
            question_resource_record_count: read_u16_be(&mut r)?,
            answer_resource_record_count: read_u16_be(&mut r)?,
            authority_resource_record_count: read_u16_be(&mut r)?,
            additional_resource_record_count: read_u16_be(&mut r)?,
        })
    }
}

#[test]
fn test_serialize() {
    let mut bin: [u8; 16] = [0; 16];
    let mut header = Header::deserialize(bin.as_slice()).unwrap();
    header.set_response();
    header.serialize(bin.as_mut_slice()).unwrap();
    println!("{bin:x?}");
}
