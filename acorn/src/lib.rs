use std::{
    fs::{File, OpenOptions},
    path::Path,
};

pub use std::io::Error;

pub struct Database {
    n: usize,
    idx: File,
    dat: File,
}

#[derive(Debug)]
struct IndexRecord {
    next_ptr: usize,
    key: Vec<u8>,
    data_len: usize,
    data_off: usize,
}

impl IndexRecord {
    pub fn empty() -> Self {
        Self {
            next_ptr: 0,
            key: vec![],
            data_len: 0,
            data_off: 0,
        }
    }
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut opts = OpenOptions::new();
        let opts = opts.read(true).write(true).create(true);
        let path = path.as_ref();
        let idx = opts.open(path.with_extension(".idx"))?;
        let dat = opts.open(path.with_extension(".dat"))?;
        let n = 3;
        let buckets = (0..=n)
            .into_iter()
            .map(|i| IndexRecord::empty())
            .collect::<Vec<_>>();
        Ok(Self { n, idx, dat })
    }

    pub fn fetch<K: AsRef<[u8]>>(&mut self, key: K) -> &[u8] {
        todo!()
    }

    pub fn store<K: AsRef<[u8]>, V: AsRef<[u8]>>(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), std::io::Error> {
        Ok(())
    }

    pub fn delete<K: AsRef<[u8]>>(&mut self, key: K) -> Result<(), Error> {
        todo!()
    }

    pub fn rewind(&mut self) -> Result<(), Error> {
        todo!()
    }

    pub fn next_rec(&mut self) -> Result<(), Error> {
        todo!()
    }

    pub fn iter(&self) -> Iter {
        todo!()
    }
}

pub struct Iter<'a> {
    database: &'a Database,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
