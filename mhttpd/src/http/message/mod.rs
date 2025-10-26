mod request;
mod response;
pub use request::{Request, RequestBuilder};
pub use response::{Response, ResponseBuilder};

use std::io::{BufRead, Write};

pub trait SerDe {
    fn read<R: BufRead>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized;
    fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}
