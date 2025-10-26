pub struct Frame {
    mem: Box<[u8]>,
}

impl Frame {
    pub fn new(page_size: usize) -> Self {
        Frame {
            mem: vec![0; page_size].into_boxed_slice(),
        }
    }

    pub fn from_bytes(buffer: Box<[u8]>) -> Self {
        Frame { mem: buffer }
    }
}

static MAGIC_STR: &str = "burqdb";
// Other details will be added later
pub struct DBHeader {
    magic_string: &'static str,
}
