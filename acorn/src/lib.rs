use std::{
    fs::{File, OpenOptions},
    path::Path,
};

pub use std::io::Error;

pub struct Database {
    idx: File,
    dat: File,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        let mut opts = OpenOptions::new();
        let opts = opts.read(true).write(true).create(true);
        let path = path.as_ref();
        let idx = opts.open(path.with_extension(".idx")).unwrap();
        let dat = opts.open(path.with_extension(".dat")).unwrap();
        Self { idx, dat }
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
