use std::io::{BufRead, Write};

pub trait Encode<Item> {
    fn encode<W: Write>(&mut self, item: Item, stream: &mut W) -> std::io::Result<()>;
}

pub trait Decode<Item> {
    fn decode<R: BufRead>(&mut self, stream: &mut R) -> std::io::Result<Item>;
}

pub trait Codec<Item>: Encode<Item> + Decode<Item> {}
