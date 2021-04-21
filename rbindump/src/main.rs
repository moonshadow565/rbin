use std::env;
use std::fs;
use rbin::*;

fn main() {
    let path = env::args().nth(1).expect("No filename");
    let file = fs::read(path).expect("Failed to read file!");
    let mut hashes = BinHashes::new();
    //hashes.fields(fs::read("hashes/hashes.binfields.txt").expect("Missing hashes.binfields.txt")).unw;

    let bin = Bin::read_from_data(file.as_slice(), &hashes).expect("Failed to read bin!");
    print!("{:#?}", bin);
}
