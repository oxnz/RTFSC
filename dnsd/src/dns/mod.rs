mod header;
mod message;
mod question;
mod record;

pub use header::{Header, ReturnCode};
pub use message::Message;
pub use question::Question;
pub use record::{ResourceRecord, ResourceRecordClass, ResourceRecordName, ResourceRecordType};
