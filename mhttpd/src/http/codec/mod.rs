use std::io::{BufRead, Write};

pub(crate) trait Encode<Item> {
    fn encode<W: Write>(&mut self, item: Item, stream: &mut W) -> std::io::Result<()>;
}

pub(crate) trait Decode<Item> {
    fn decode<R: BufRead>(&mut self, stream: &mut R) -> std::io::Result<Item>;
}

pub(crate) trait Codec<Item>: Encode<Item> + Decode<Item> {}

pub(crate) trait SerDe {
    fn read<R: BufRead>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized;
    fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}
