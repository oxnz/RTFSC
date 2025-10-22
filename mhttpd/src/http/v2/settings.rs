use crate::http::SerDe;

/**
 * 0x1
SETTINGS_HEADER_TABLE_SIZE
0x2
SETTINGS_ENABLE_PUSH
0x3
SETTINGS_MAX_CONCURRENT_STREAMS
0x4
SETTINGS_INITIAL_WINDOW_SIZE
0x5
SETTINGS_MAX_FRAME_SIZE
0x6
SETTINGS_MAX_HEADER_LIST_SIZE
 */
#[repr(u16)]
#[derive(Debug, Clone, Copy)]
pub enum Setting {
    HeaderTableSize(u32),
    EnablePush(bool),
    MaxConcurrentStreams(u32),
    InitialWindowSize(u32),
    MaxFrameSize(u32),
    MaxHeaderListSize(u32),
}

impl SerDe for Setting {
    fn read<R: std::io::BufRead>(r: &mut R) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut identifier = [0u8; 2];
        r.read_exact(&mut identifier)?;
        let mut value = [0u8; 4];
        r.read_exact(&mut value)?;
        let value = u32::from_be_bytes(value);
        match u16::from_be_bytes(identifier) {
            0x01 => Ok(Self::HeaderTableSize(value)),
            0x02 => Ok(Self::EnablePush(value != 0)),
            0x03 => Ok(Self::MaxConcurrentStreams(value)),
            0x04 => Ok(Self::InitialWindowSize(value)),
            0x05 => Ok(Self::MaxFrameSize(value)),
            0x06 => Ok(Self::MaxHeaderListSize(value)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid identifier",
            )),
        }
    }

    fn write<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        match self {
            Setting::HeaderTableSize(value) => {
                w.write_all(&0x01u16.to_be_bytes())?;
                w.write_all(&value.to_be_bytes())
            }
            Setting::EnablePush(value) => {
                w.write_all(&0x02u16.to_be_bytes())?;
                let value: u32 = if *value { 1 } else { 0 };
                w.write_all(&value.to_be_bytes())
            }
            Setting::MaxConcurrentStreams(value) => {
                w.write_all(&0x03u16.to_be_bytes())?;
                w.write_all(&value.to_be_bytes())
            }
            Setting::InitialWindowSize(value) => {
                w.write_all(&0x04u16.to_be_bytes())?;
                w.write_all(&value.to_be_bytes())
            }
            Setting::MaxFrameSize(value) => {
                w.write_all(&0x05u16.to_be_bytes())?;
                w.write_all(&value.to_be_bytes())
            }
            Setting::MaxHeaderListSize(value) => {
                w.write_all(&0x06u16.to_be_bytes())?;
                w.write_all(&value.to_be_bytes())
            }
        }
    }
}

#[test]
fn test_serde() {
    let raw = [0, 3, 0, 0, 0, 0x64, 0, 4, 0, 0xa0, 0, 0, 0, 2, 0, 0, 0, 0];
    let mut buf = Vec::new();

    for item in unsafe { raw.as_chunks_unchecked::<6>() } {
        let setting = Setting::read(&mut std::io::Cursor::new(item)).unwrap();
        println!("{setting:?}");
        setting.write(&mut buf).unwrap();
    }
    assert_eq!(buf, raw);
}
