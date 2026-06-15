use flate2::{write::ZlibEncoder, Compression};
use std::io::Write;

pub struct NbtBuf(pub Vec<u8>);

impl NbtBuf {
    pub fn new() -> Self { Self(Vec::new()) }

    fn u8(&mut self, v: u8) { self.0.push(v); }
    fn be16(&mut self, v: u16) { self.0.extend_from_slice(&v.to_be_bytes()); }
    fn be32(&mut self, v: u32) { self.0.extend_from_slice(&v.to_be_bytes()); }
    fn be64(&mut self, v: u64) { self.0.extend_from_slice(&v.to_be_bytes()); }

    fn tag_header(&mut self, tag: u8, name: &str) {
        self.u8(tag);
        self.be16(name.len() as u16);
        self.0.extend_from_slice(name.as_bytes());
    }

    pub fn byte(&mut self, name: &str, v: i8) { self.tag_header(1, name); self.u8(v as u8); }
    pub fn short(&mut self, name: &str, v: i16) { self.tag_header(2, name); self.be16(v as u16); }
    pub fn int(&mut self, name: &str, v: i32) { self.tag_header(3, name); self.be32(v as u32); }
    pub fn long(&mut self, name: &str, v: i64) { self.tag_header(4, name); self.be64(v as u64); }
    pub fn string(&mut self, name: &str, v: &str) {
        self.tag_header(8, name);
        self.be16(v.len() as u16);
        self.0.extend_from_slice(v.as_bytes());
    }
    pub fn byte_array(&mut self, name: &str, data: &[u8]) {
        self.tag_header(7, name);
        self.be32(data.len() as u32);
        self.0.extend_from_slice(data);
    }
    pub fn int_array(&mut self, name: &str, data: &[i32]) {
        self.tag_header(11, name);
        self.be32(data.len() as u32);
        for v in data { self.be32(*v as u32); }
    }
    pub fn begin_compound(&mut self, name: &str) { self.tag_header(10, name); }
    pub fn end_compound(&mut self) { self.u8(0); }
    pub fn begin_list(&mut self, name: &str, elem_tag: u8, len: i32) {
        self.tag_header(9, name);
        self.u8(elem_tag);
        self.be32(len as u32);
    }
    pub fn begin_list_compound_element(&mut self) {}
    pub fn end_list_compound_element(&mut self) { self.u8(0); }
}

pub fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::best());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}
