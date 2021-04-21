mod hashes;
mod reader;

pub use hashes::*;
use reader::BinReader;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum BinValue {
    None,
    Bool(bool),
    Signed(i64),
    Unsigned(u64),
    Float(f32),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mtx44([[f32; 4]; 4]),
    Rgba([u8; 4]),
    String(String),
    Hash(BinFNV),
    Link(BinFNV),
    File(BinXXH),
    List(Vec<BinValue>),
    Map(Vec<(BinValue, BinValue)>),
    Struct(BinFNV, HashMap<BinFNV, BinValue>),
}

#[derive(Clone, Debug)]
pub struct Bin {
    pub version: u32,
    pub links: Vec<String>,
    pub entries: HashMap<BinFNV, BinValue>,
}

impl Bin {
    pub fn read_from(data: &[u8], hashes: &BinHashes) -> std::io::Result<Bin> {
        BinReader::read_bin(data, hashes)
    }
}