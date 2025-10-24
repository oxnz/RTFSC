pub mod v1;
pub mod v2;

use bytes::BytesMut;

pub trait Encode<Item> {
    fn encode(&mut self, item: Item, stream: &mut BytesMut) -> std::io::Result<()>;
}

pub trait Decode<Item> {
    fn decode(&mut self, stream: &mut BytesMut) -> std::io::Result<Option<Item>>;
}
