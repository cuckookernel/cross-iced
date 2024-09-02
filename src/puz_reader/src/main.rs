mod import_puz;
use crate::import_puz::import_puzzle;

use std::fs::File;
use std::io::BufReader;


fn main() {
    let path = "/home/teo/Downloads/wsj240702.puz";
    let mut f = BufReader::new(File::open(path).unwrap());

    // let header = read_header(&mut f).unwrap();
    let imported_puz = import_puzzle(&mut f).unwrap();
    println!("Imported_puz: {imported_puz:?}")
}
