use std::io::{Read, Write};

use crate::{SerDe, read_u16_be, write_u16_be};

#[derive(Debug)]
pub struct Header {
    /** A 16-bit field identifying a specific DNS transaction.
     * The transaction ID is created by the message originator and
     * is copied by the responder into its response message.
     * Using the transaction ID, the DNS client can match responses to its requests.
     */
    pub transaction_id: u16,
    /**
     * QR: 1 => 0 for query, 1 for response
     * OPCODE: 4
     * AA: 1
     * TC: 1
     * RD: 1
     * --
     * RA: 1
     * Z: 3
     * RCODE: 4
     */
    pub flags: u16,
    pub question_resource_record_count: u16,
    pub answer_resource_record_count: u16,
    pub authority_resource_record_count: u16,
    pub additional_resource_record_count: u16,
}

#[repr(u8)]
#[derive(Debug)]
pub enum ReturnCode {
    NoError = 0x00,
    FormatError = 0x01,
    ServerFailure = 0x02,
    NameError = 0x03,
    NotImplemented = 0x04,
    Refused = 0x05,
    /// A name that should not exist does exist.
    YXDOMAIN = 0x06,
    ///  A resource record set that should not exist does exist.
    YXRRSET = 0x07,
    /// A resource record set that should exist does not exist.
    NXRRSET = 0x8,
    /// DNS server is not authoritative for the zone named in the Zone section.
    NOTAUTH = 0x09,
    /// A name used in the Prerequisite or Update sections is not within the zone specified by the Zone section.
    NOTZONE = 0x0A,
    Reserved,
}

impl Header {
    pub fn set_request(&mut self) {
        self.flags &= !0x8000;
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

    pub fn set_rcode(&mut self, code: ReturnCode) {
        self.flags |= (0x0F & code as u8) as u16;
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
