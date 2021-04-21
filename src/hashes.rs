use num_traits::{Num, Unsigned};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};

pub trait BinHashed: Clone + Debug + Eq + Ord + Hash {
    type HashType: Num + Unsigned + Copy + Display + Debug + Eq + Ord + Hash;

    fn from_hash(hash: Self::HashType) -> Self
    where
        Self: Sized;

    fn from_string(string: &str) -> Self
    where
        Self: Sized;

    fn from_hash_string(hash: Self::HashType, string: &str) -> Self
    where
        Self: Sized;

    fn get_hash(&self) -> Self::HashType;

    fn get_string(&self) -> &str;

    fn format_to(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

#[derive(Clone)]
pub struct BinFNV {
    hash: u32,
    unhashed: String,
}

impl BinHashed for BinFNV {
    type HashType = u32;

    fn from_hash(hash: Self::HashType) -> Self {
        Self {
            hash,
            unhashed: String::new(),
        }
    }

    fn from_string(string: &str) -> Self {
        let mut hash = 0x811c9dc5u32;
        for c in string.to_ascii_lowercase().as_bytes() {
            hash = hash ^ (*c as u32);
            hash = hash.wrapping_mul(0x01000193u32);
        }
        Self {
            hash,
            unhashed: String::new(),
        }
    }

    fn from_hash_string(hash: Self::HashType, string: &str) -> Self {
        Self {
            hash,
            unhashed: string.to_string(),
        }
    }

    fn get_hash(&self) -> Self::HashType {
        self.hash
    }

    fn get_string(&self) -> &str {
        &self.unhashed
    }

    fn format_to(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result  {
        if self.unhashed.len() != 0 {
            write!(f, "{:?}", self.unhashed)
        } else {
            write!(f, "0x{:08X}", self.hash)
        }
    }
}

impl Display for BinFNV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_to(f)
    }
}

impl Debug for BinFNV {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_to(f)
    }
}

impl Hash for BinFNV {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_hash().hash(state)
    }
}

impl PartialEq for BinFNV {
    fn eq(&self, other: &Self) -> bool {
        self.get_hash() == other.get_hash()
    }
}

impl Eq for BinFNV {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for BinFNV {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl Ord for BinFNV {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

#[derive(Clone)]
pub struct BinXXH {
    hash: u64,
    unhashed: String,
}

impl BinHashed for BinXXH {
    type HashType = u64;

    fn from_hash(hash: Self::HashType) -> Self {
        Self {
            hash,
            unhashed: String::new(),
        }
    }

    fn from_string(string: &str) -> Self {
        todo!("Implement this")
    }

    fn from_hash_string(hash: Self::HashType, string: &str) -> Self {
        Self {
            hash,
            unhashed: string.to_string(),
        }
    }

    fn get_hash(&self) -> Self::HashType {
        self.hash
    }

    fn get_string(&self) -> &str {
        &self.unhashed
    }

    fn format_to(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result  {
        if self.unhashed.len() != 0 {
            write!(f, "{:?}", self.unhashed)
        } else {
            write!(f, "0x{:016X}", self.hash)
        }
    }
}

impl Display for BinXXH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_to(f)
    }
}

impl Debug for BinXXH {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.format_to(f)
    }
}

impl Hash for BinXXH {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.get_hash().hash(state)
    }
}

impl PartialEq for BinXXH {
    fn eq(&self, other: &Self) -> bool {
        self.get_hash() == other.get_hash()
    }
}

impl Eq for BinXXH {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for BinXXH {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl Ord for BinXXH {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

pub struct BinHashList<T>
where
    T: BinHashed,
{
    pub list: HashMap<T::HashType, String>,
}

impl<T> BinHashList<T>
where
    T: BinHashed,
{
    pub fn new() -> Self {
        Self {
            list: HashMap::new(),
        }
    }

    pub fn read_from_file(&mut self, file: File) -> Result<(), String> {
        for line in BufReader::new(file).lines().into_iter() {
            let line = line.map_err(|_| "Failed to read line".to_string())?;
            let (hash, unhashed) = match line.split_once(" ") {
                Some((hash_hex, hash_str)) => {
                    match T::HashType::from_str_radix(&hash_hex, 16) {
                        Ok(hash) => Ok((hash, hash_str.to_string())),
                        _ => Err("Failed to convert hex".to_string()),
                    }
                },
                None => Err("Each line must contain a space".to_string()),
            }?;
            self.list.insert(hash, unhashed);
        }
        Ok(())
    }

    pub fn get(&self, hash: T::HashType) -> T {
        if let Some(string) = self.list.get(&hash) {
            T::from_hash_string(hash, string)
        } else {
            T::from_hash(hash)
        }
    }
}

pub struct BinHashes {
    pub entries: BinHashList<BinFNV>,
    pub fields: BinHashList<BinFNV>,
    pub hashes: BinHashList<BinFNV>,
    pub types: BinHashList<BinFNV>,
    pub paths: BinHashList<BinXXH>,
}
impl BinHashes {
    pub fn new() -> BinHashes {
        BinHashes {
            entries: BinHashList::new(),
            fields: BinHashList::new(),
            hashes: BinHashList::new(),
            types: BinHashList::new(),
            paths: BinHashList::new(),
        }
    }
}
