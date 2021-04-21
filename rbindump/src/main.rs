use rbin::*;
use std::env;
use std::fs;
use std::fs::File;

fn main() {
    let path = env::args().nth(1).expect("No filename");
    let file = fs::read(path).expect("Failed to read file!");
    let mut hashes = BinHashes::new();
    hashes.fields.read_from_file(File::open("hashes/hashes.binfields.txt").expect("Missing hashes.binfields.txt")).unwrap();
    hashes.types.read_from_file(File::open("hashes/hashes.bintypes.txt").expect("Missing hashes.bintypes.txt")).unwrap();
    hashes.hashes.read_from_file(File::open("hashes/hashes.binhashes.txt").expect("Missing hashes.binhashes.txt")).unwrap();
    hashes.entries.read_from_file(File::open("hashes/hashes.binentries.txt").expect("Missing hashes.binentries.txt")).unwrap();
    hashes.paths.read_from_file(File::open("hashes/hashes.game.txt").expect("Missing hashes.game.txt")).unwrap();
    let bin = Bin::read_from_data(file.as_slice(), &hashes).expect("Failed to read bin!");
    print!("{:#?}", bin);
}
