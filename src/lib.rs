mod hashes;
mod reader;

pub use hashes::*;
use reader::BinReader;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::fmt::{Debug, Display};

#[derive(Clone)]
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

impl BinValue {
    pub fn format_to(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinValue::None => write!(f, "None"),
            BinValue::Bool(value) => write!(f, "{}", value),
            BinValue::Signed(value) => write!(f, "{}", value),
            BinValue::Unsigned(value) => write!(f, "{}", value),
            BinValue::Float(value) => write!(f, "{}", value),
            BinValue::Vec2(value) => write!(f, "{:?}", value),
            BinValue::Vec3(value) => write!(f, "{:?}", value),
            BinValue::Vec4(value) => write!(f, "{:?}", value),
            BinValue::Mtx44(value) =>  write!(f, "{:?}", value),
            BinValue::Rgba(value) => write!(f, "[ 0x{:02X} {:02X}, 0x{:02X}, 0x{:02X}, ]",  value[0], value[1], value[2], value[3]),
            BinValue::String(value) =>  write!(f, "{:?}", value),
            BinValue::Hash(value) => value.format_to(f),
            BinValue::Link(value) => value.format_to(f),
            BinValue::File(value) => value.format_to(f),
            BinValue::List(value) =>  write!(f, "{:#?}", value),
            BinValue::Map(value) => {
                let mut debug = f.debug_map();
                for (key, value) in value {
                    debug.key(key);
                    debug.value(value);
                }
                debug.finish()
            },
            BinValue::Struct(name, fields) => {
                let name = if name.get_string().len() != 0 {
                    name.get_string().to_string()
                } else {
                    format!("0x{:08X}", name.get_hash())
                };
                let mut debug = f.debug_struct(&name);
                for (name, value) in fields {
                    let name = if name.get_string().len() != 0 {
                        name.get_string().to_string()
                    } else {
                        format!("0x{:08X}", name.get_hash())
                    };
                    debug.field(&name, value);
                }
                debug.finish()
            }
        }
    }
}

impl Display for BinValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_to(f)
    }
}

impl Debug for BinValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_to(f)
    }
}


#[derive(Clone, Debug)]
pub struct Bin {
    pub version: u32,
    pub links: Vec<String>,
    pub entries: HashMap<BinFNV, BinValue>,
}

impl Bin {
    pub fn read_from_data(data: &[u8], hashes: &BinHashes) -> std::io::Result<Bin> {
        BinReader::read_bin(data, hashes)
    }

    pub fn read_from_file(file: File, hashes: &BinHashes) -> std::io::Result<Bin> {
        let mut file = file;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Self::read_from_data(buf.as_slice(), hashes)
    }
}
