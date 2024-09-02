use std::collections::{BTreeSet, BTreeMap};
use std::io::{self, BufRead};
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
};

use crate::api_types::{Pos, TypingDir};

#[derive(Debug, Default)]
pub struct Header {
    cksum: u16, // len 2
    // file_magic: [u8; 0xc], // len 0xC = 12
    file_magic: String,
    cib_cksum: u16, // len 2
    masked_low_cksums: [u8; 4],
    masked_high_cksums: [u8; 4],

    ver_str: String, // len 4
    reserved_1c: [u8; 2],
    scrambled_cksum: u16,
    width: usize,
    height: usize,
    num_clues: usize, // size 2
    unk_bitmask: u16,
    scrambled_tag: u16,
}

#[derive(Debug, Default)]
pub struct CellNumbers {
    cell_2_num: BTreeMap<Pos, usize>,
    across_numbers: BTreeSet<usize>,
    down_numbers: BTreeSet<usize>,
}

#[derive(Debug, Default)]
pub struct Pos2ClueIdx(pub BTreeMap<(Pos, TypingDir), usize>);


#[allow(unused)]
#[derive(Debug, Default)]
pub struct BoardContents {
    data: Vec<Vec<char>>,
}

impl BoardContents {
    fn calc_cell_numbers(&self) -> CellNumbers {
        let height = self.data.len();
        let width = self.data[0].len();

        let mut ret = CellNumbers::default();
        let mut cur_cell_number = 1usize;

        for r_idx in 0..height {
            // let mut cell_number_row: Vec<Option<usize>> = vec![None; width];

            for c_idx in 0..width {
                if self.is_black_cell(r_idx, c_idx) {
                    continue
                }

                let mut assigned_number = false;

                if self.cell_needs_across_number(r_idx, c_idx) {
                    ret.across_numbers.insert(cur_cell_number);
                    ret.cell_2_num.insert(Pos::new(r_idx, c_idx), cur_cell_number);
                    assigned_number = true
                }

                if self.cell_needs_down_number(r_idx, c_idx) {
                    ret.down_numbers.insert(cur_cell_number);
                    ret.cell_2_num.insert(Pos::new(r_idx, c_idx), cur_cell_number);
                    assigned_number = true
                }

                if assigned_number {
                    cur_cell_number += 1
                }
            }
        }

        ret
    }

    fn cell_needs_across_number(&self, r_idx: usize, c_idx: usize) -> bool {
        (c_idx == 0 || self.is_black_cell(r_idx, c_idx - 1))
            && (c_idx + 1 < self.width() && !self.is_black_cell(r_idx, c_idx + 1))
    }

    fn cell_needs_down_number(&self, r_idx: usize, c_idx: usize) -> bool {
        (r_idx == 0 || self.is_black_cell(r_idx - 1, c_idx))
            && (r_idx + 1 < self.height() && !self.is_black_cell(r_idx + 1, c_idx))
    }

    fn is_black_cell(&self, r_idx: usize, c_idx: usize ) ->  bool {
        self.data[r_idx][c_idx] == '.'
    }

    fn height(&self) -> usize {
        self.data.len()
    }

    fn width(&self) -> usize {
        self.data[0].len()
    }

}


impl CellNumbers{
    fn calc_num_2_clue_idx(&self) -> Pos2ClueIdx {
        let mut clue_idx = 0;
        let mut all_numbers: BTreeSet<usize> = self.across_numbers.union(&self.down_numbers).cloned().collect();
            
        let num_2_pos: BTreeMap<usize, Pos> = self.cell_2_num
            .iter()
            .map(|(pos, num)| (*num, *pos)).collect();
            
        let mut ret = BTreeMap::new();
        while !all_numbers.is_empty() {
            let num = all_numbers.pop_first().unwrap();
            if self.across_numbers.contains(&num) {
                let pos = num_2_pos.get(&num).expect(format!("No pos for num={num}").as_str());
                ret.insert( (*pos, TypingDir::Across), clue_idx );
                println!("Assigned num: {num}, dir: across  clue_idx: {clue_idx}");
                clue_idx += 1 
            }
            
            if self.down_numbers.contains(&num) {
                let pos = num_2_pos.get(&num).expect(format!("No pos for num={num}").as_str());
                ret.insert( (*pos, TypingDir::Down), clue_idx );
                println!("Assigned num: {num}, dir: down  clue_idx: {clue_idx}");
                clue_idx += 1
            }
        }
        
        Pos2ClueIdx(ret)
    }
}


#[allow(unused)]
#[derive(Debug, Default)]
pub struct PuzStrings {
    title: String,
    author: String,
    copyright: String,
    clues: Vec<String>,
    notes: String,
}

#[allow(unused)]
#[derive(Debug, Default)]
pub struct ImportedPuz {
    header: Header,
    solution: BoardContents,
    player_state: BoardContents,
    strings: PuzStrings,
    pub pos_2_clue_idx: Pos2ClueIdx, 
}

impl ImportedPuz {
    pub fn width(&self) -> usize {
        self.header.width
    }

    pub fn height(&self) -> usize {
        self.header.height
    }

    pub fn solution_at(&self, r_idx: usize, c_idx: usize) -> char {
        self.solution.data[r_idx][c_idx]
    }
    
    pub fn clues(&self) -> &Vec<String> {
        &self.strings.clues
    }
}

pub fn import_puzzle(f: &mut BufReader<File>) -> io::Result<ImportedPuz> {
    let header = read_header(f)?;
    let (width, height, n_clues) = (header.width, header.height, header.num_clues);
    let solution = read_contents(f, height, width)?;

    for r_idx in 0..height {
        println!("row {:02} : {s:?}", r_idx, s=solution.data[r_idx]);
    }

    let pos_2_num: CellNumbers = solution.calc_cell_numbers();
    println!("pos_2_num: {pos_2_num:?}");
    let pos_2_clue_idx = pos_2_num.calc_num_2_clue_idx();
    
    Ok(ImportedPuz {
        header,
        solution,
        player_state: read_contents(f, height, width)?,
        strings: read_strings(f, n_clues)?,
        pos_2_clue_idx
    })
}

pub fn read_header(f: &mut BufReader<File>) -> io::Result<Header> {
    let mut header = Header::default();

    header.cksum = read_u16(f)?;
    header.file_magic = read_latin1_string(f, 0xc)?;
    header.cib_cksum = read_u16(f)?;

    f.read_exact(&mut header.masked_low_cksums)?;
    f.read_exact(&mut header.masked_high_cksums)?;

    header.ver_str = read_latin1_string(f, 0x4)?;
    f.read_exact(&mut header.reserved_1c)?;

    header.scrambled_cksum = read_u16(f)?;

    f.seek(SeekFrom::Start(0x2c))?;
    header.width = read_u8(f)? as usize;
    header.height = read_u8(f)? as usize;

    let pos = f.stream_position()?;
    println!("After reading height: pos={pos:#02x}");
    header.num_clues = read_u16(f)? as usize;

    println!("num_clues={}", header.num_clues);

    header.unk_bitmask = read_u16(f)?;
    header.scrambled_tag = read_u16(f)?;

    let pos = f.stream_position()?;
    println!("After reading scrambled_tag: pos={pos:#02x}");

    Ok(header)
}

pub fn read_contents(
    f: &mut BufReader<File>,
    height: usize,
    width: usize,
) -> io::Result<BoardContents> {
    let mut data: Vec<Vec<char>> = Vec::new();

    for _ in 0..height {
        let row = read_len(f, width)?;
        data.push(row.into_iter().map(|by| by as char).collect())
    }

    Ok(BoardContents { data })
}

pub fn read_strings(f: &mut BufReader<File>, num_clues: usize) -> io::Result<PuzStrings> {
    // let mut rest = String::new();
    // f.read_to_string(&mut rest)?;

    let mut parts: VecDeque<String> = VecDeque::new();
    let mut buf = Vec::<u8>::new();

    for _ in 0..num_clues + 4 {
        let num_read = f.read_until(b'\0', &mut buf)?;
        if num_read == 0 {
            return Err(io::ErrorKind::UnexpectedEof.into())
        }

        let a_str: String = buf[..buf.len() - 1].iter().map(|by| *by as char).collect();
        parts.push_back(a_str);
        buf.clear();
    }

    let title = parts.pop_front().unwrap();
    let author = parts.pop_front().unwrap();
    let copyright =  parts.pop_front().unwrap();
    let notes = parts.pop_back().unwrap();
    let clues =  parts.iter().map(|s| s.to_string()).collect();

    Ok(PuzStrings {
        title,
        author,
        copyright,
        clues,
        notes
    })
}



fn read_u16(f: &mut BufReader<File>) -> io::Result<u16> {
    let mut short_buf = [0; 2];
    f.read_exact(&mut short_buf)?;
    Ok(u16::from_le_bytes(short_buf))
}

fn read_u8(f: &mut BufReader<File>) -> io::Result<u8> {
    let mut one_byte = [0; 1];
    f.read_exact(&mut one_byte)?;
    Ok(one_byte[0])
}

fn read_latin1_string(f: &mut BufReader<File>, len: usize) -> io::Result<String> {
    let bytes = read_len(f, len)?;
    Ok(latin1_to_str(&bytes))
}

fn read_len(f: &mut BufReader<File>, len: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

fn latin1_to_str(bytes: &[u8]) -> String {
    bytes.iter().map(|&by| by as char).collect()
}

