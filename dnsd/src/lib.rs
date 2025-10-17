use std::io::{Read, Write};

pub mod dns;
pub mod server;

pub trait SerDe {
    fn serialize<W: Write>(&self, w: W) -> std::io::Result<()>;

    fn deserialize<R: Read>(r: R) -> std::io::Result<Self>
    where
        Self: Sized;
}

pub(crate) fn read_u16_be<R: Read>(r: &mut R) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

pub(crate) fn read_u32_be<R: Read>(r: &mut R) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

pub(crate) fn write_u16_be<W: Write>(w: &mut W, value: u16) -> std::io::Result<()> {
    let bytes = value.to_be_bytes();
    w.write(&bytes)?;
    Ok(())
}

pub(crate) fn write_u32_be<W: Write>(w: &mut W, value: u32) -> std::io::Result<()> {
    let bytes = value.to_be_bytes();
    w.write(&bytes)?;
    Ok(())
}
