use crate::*;
use num_enum::TryFromPrimitive;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};

#[derive(TryFromPrimitive, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
#[repr(u8)]
enum BinType {
    None = 0,
    Bool = 1,
    I8 = 2,
    U8 = 3,
    I16 = 4,
    U16 = 5,
    I32 = 6,
    U32 = 7,
    I64 = 8,
    U64 = 9,
    F32 = 10,
    Vec2 = 11,
    Vec3 = 12,
    Vec4 = 13,
    Mtx44 = 14,
    Rgba = 15,
    String = 16,
    Hash = 17,
    File = 18,
    List = 0x80 | 0,
    List2 = 0x80 | 1,
    Pointer = 0x80 | 2,
    Embed = 0x80 | 3,
    Link = 0x80 | 4,
    Option = 0x80 | 5,
    Map = 0x80 | 6,
    Flag = 0x80 | 7,
}

pub struct BinReader<'a, 'b> {
    cur: Cursor<&'a [u8]>,
    depth: usize,
    hashes: &'b BinHashes,
}

impl<'a, 'b> BinReader<'a, 'b> {
    fn read_i8(&mut self) -> Result<i8> {
        let mut buffer = [0; 1];
        self.cur.read_exact(&mut buffer)?;
        Ok(i8::from_le_bytes(buffer))
    }

    fn read_u8(&mut self) -> Result<u8> {
        let mut buffer = [0; 1];
        self.cur.read_exact(&mut buffer)?;
        Ok(u8::from_le_bytes(buffer))
    }

    fn read_i16(&mut self) -> Result<i16> {
        let mut buffer = [0; 2];
        self.cur.read_exact(&mut buffer)?;
        Ok(i16::from_le_bytes(buffer))
    }

    fn read_u16(&mut self) -> Result<u16> {
        let mut buffer = [0; 2];
        self.cur.read_exact(&mut buffer)?;
        Ok(u16::from_le_bytes(buffer))
    }

    fn read_i32(&mut self) -> Result<i32> {
        let mut buffer = [0; 4];
        self.cur.read_exact(&mut buffer)?;
        Ok(i32::from_le_bytes(buffer))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let mut buffer = [0; 4];
        self.cur.read_exact(&mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_i64(&mut self) -> Result<i64> {
        let mut buffer = [0; 8];
        self.cur.read_exact(&mut buffer)?;
        Ok(i64::from_le_bytes(buffer))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let mut buffer = [0; 8];
        self.cur.read_exact(&mut buffer)?;
        Ok(u64::from_le_bytes(buffer))
    }

    fn read_f32(&mut self) -> Result<f32> {
        let mut buffer = [0; 4];
        self.cur.read_exact(&mut buffer)?;
        Ok(f32::from_le_bytes(buffer))
    }

    fn read_type(&mut self) -> Result<BinType> {
        Ok(BinType::try_from(self.read_u8()?).unwrap())
    }

    fn read_vec2(&mut self) -> Result<[f32; 2]> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        Ok([x, y])
    }

    fn read_vec3(&mut self) -> Result<[f32; 3]> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        Ok([x, y, z])
    }

    fn read_vec4(&mut self) -> Result<[f32; 4]> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        let w = self.read_f32()?;
        Ok([x, y, z, w])
    }

    fn read_mtx44(&mut self) -> Result<[[f32; 4]; 4]> {
        let r0 = self.read_vec4()?;
        let r1 = self.read_vec4()?;
        let r2 = self.read_vec4()?;
        let r3 = self.read_vec4()?;
        Ok([r0, r1, r2, r3])
    }

    fn read_rgba(&mut self) -> Result<[u8; 4]> {
        let a = self.read_u8()?;
        let b = self.read_u8()?;
        let g = self.read_u8()?;
        let r = self.read_u8()?;
        Ok([r, g, b, a])
    }

    fn read_string(&mut self) -> Result<String> {
        let len = self.read_u16()?;
        let cur_pos = self.cur.position() as usize;
        let end_pos = self.cur.seek(SeekFrom::Current(len as i64))? as usize;
        let mut cur = Cursor::new(&self.cur.get_ref()[cur_pos..end_pos]);
        let mut result = String::new();
        cur.read_to_string(&mut result)?;
        Ok(result)
    }

    fn read_hash_name(&mut self) -> Result<BinFNV> {
        let hash = self.read_u32()?;
        Ok(self.hashes.hashes.get(hash))
    }

    fn read_entry_name(&mut self) -> Result<BinFNV> {
        let hash = self.read_u32()?;
        Ok(self.hashes.entries.get(hash))
    }

    fn read_type_name(&mut self) -> Result<BinFNV> {
        let hash = self.read_u32()?;
        Ok(self.hashes.types.get(hash))
    }

    fn read_field_name(&mut self) -> Result<BinFNV> {
        let hash = self.read_u32()?;
        Ok(self.hashes.fields.get(hash))
    }

    fn read_path_name(&mut self) -> Result<BinXXH> {
        let hash = self.read_u64()?;
        Ok(self.hashes.paths.get(hash))
    }

    fn read_sub_reader(&mut self) -> Result<BinReader<'a, 'b>> {
        let depth = self.depth + 1;
        if depth > 128 {
            Err(Error::new(
                ErrorKind::Other,
                "Sub reader depth limit reached",
            ))
        } else {
            let len = self.read_u32()? as i64;
            let cur_pos = self.cur.position();
            let end_pos = self.cur.seek(SeekFrom::Current(len))? as usize;
            let mut cur = Cursor::new(&self.cur.get_ref()[..end_pos]);
            cur.set_position(cur_pos);
            Ok(BinReader {
                cur,
                depth,
                hashes: self.hashes,
            })
        }
    }

    fn read_fields(&mut self) -> Result<HashMap<BinFNV, BinValue>> {
        let mut result = HashMap::new();
        let count = self.read_u16()?;
        for _ in 0..count {
            let key = self.read_field_name()?;
            let value_type = self.read_type()?;
            let value = self.read_value(value_type)?;
            result.insert(key, value);
        }
        Ok(result)
    }

    fn read_value(&mut self, bin_type: BinType) -> Result<BinValue> {
        let io = self;
        Ok(match bin_type {
            BinType::None => BinValue::None,
            BinType::Bool | BinType::Flag => BinValue::Bool(io.read_u8()? != 0),
            BinType::I8 => BinValue::Signed(io.read_i8()? as i64),
            BinType::U8 => BinValue::Unsigned(io.read_u8()? as u64),
            BinType::I16 => BinValue::Signed(io.read_i16()? as i64),
            BinType::U16 => BinValue::Unsigned(io.read_u16()? as u64),
            BinType::I32 => BinValue::Signed(io.read_i32()? as i64),
            BinType::U32 => BinValue::Unsigned(io.read_u32()? as u64),
            BinType::I64 => BinValue::Signed(io.read_i64()? as i64),
            BinType::U64 => BinValue::Unsigned(io.read_u64()? as u64),
            BinType::F32 => BinValue::Float(io.read_f32()?),
            BinType::Vec2 => BinValue::Vec2(io.read_vec2()?),
            BinType::Vec3 => BinValue::Vec3(io.read_vec3()?),
            BinType::Vec4 => BinValue::Vec4(io.read_vec4()?),
            BinType::Mtx44 => BinValue::Mtx44(io.read_mtx44()?),
            BinType::Rgba => BinValue::Rgba(io.read_rgba()?),
            BinType::String => BinValue::String(io.read_string()?),
            BinType::Hash => BinValue::Hash(io.read_hash_name()?),
            BinType::Link => BinValue::Link(io.read_entry_name()?),
            BinType::File => BinValue::File(io.read_path_name()?),
            BinType::Option => {
                let value_type = io.read_type()?;
                let count = io.read_u8()?;
                if count == 0 {
                    BinValue::None
                } else {
                    io.read_value(value_type)?
                }
            }
            BinType::List | BinType::List2 => {
                let value_type = io.read_type()?;
                let mut io = io.read_sub_reader()?;
                let count = io.read_u32()?;
                let mut result = Vec::new();
                for _ in 0..count {
                    result.push(io.read_value(value_type)?)
                }
                BinValue::List(result)
            }
            BinType::Map => {
                let key_type = io.read_type()?;
                let value_type = io.read_type()?;
                let mut io = io.read_sub_reader()?;
                let count = io.read_u32()?;
                let mut result = Vec::new();
                for _ in 0..count {
                    let key = io.read_value(key_type)?;
                    let value = io.read_value(value_type)?;
                    result.push((key, value))
                }
                BinValue::Map(result)
            }
            BinType::Pointer | BinType::Embed => {
                let type_name = io.read_type_name()?;
                if type_name.get_hash() == 0 {
                    BinValue::None
                } else {
                    let mut io = io.read_sub_reader()?;
                    let fields = io.read_fields()?;
                    BinValue::Struct(type_name, fields)
                }
            }
        })
    }

    fn read_entries(&mut self) -> Result<HashMap<BinFNV, BinValue>> {
        let count = self.read_u32()?;
        let mut type_names = Vec::new();
        for _ in 0..count {
            let type_name = self.read_type_name()?;
            type_names.push(type_name);
        }
        let mut result = HashMap::new();
        for type_name in type_names {
            let mut io = self.read_sub_reader()?;
            let key = io.read_entry_name()?;
            let fields = io.read_fields()?;
            let value = BinValue::Struct(type_name, fields);
            result.insert(key, value);
        }
        Ok(result)
    }

    fn read_links(&mut self) -> Result<Vec<String>> {
        let count = self.read_u32()?;
        let mut result = Vec::new();
        for _ in 0..count {
            let value = self.read_string()?;
            result.push(value);
        }
        Ok(result)
    }

    pub fn read_bin(data: &[u8], hashes: &BinHashes) -> Result<Bin> {
        let cur = Cursor::new(data);
        let mut reader = BinReader {
            cur,
            depth: 0,
            hashes,
        };
        let magic = reader.read_u32()?;
        if magic == 0x504f5250 {
            let version = reader.read_u32()?;
            let links = reader.read_links()?;
            let entries = reader.read_entries()?;
            Ok(Bin {
                version,
                links,
                entries,
            })
        } else {
            Err(Error::new(ErrorKind::Other, "Bad bin magic"))
        }
    }
}
