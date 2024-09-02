use std::collections::BTreeSet;

use crate::{api_types::*, import_puz:: Pos2ClueIdx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    OccupiedRight(char),
    OccupiedWrong(char),
    Black,
}

impl Cell {
    fn get_char(&self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Black => ' ',
            Self::OccupiedRight(c) => *c,
            Self::OccupiedWrong(c) => *c
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SelectedWord {
    poss: BTreeSet<Pos>
}

impl SelectedWord {
    pub fn contains(&self, pos: &Pos) -> bool {
        self.poss.contains(pos)
    }
    pub fn first_pos(&self) -> Option<Pos> {
        self.poss.first().cloned()
    }
}

pub struct Board {
    pub content: Vec<Vec<Cell>>,
    pub solution: Vec<Vec<Cell>>,
    pub clues: Vec<String>,
    pub pos_2_clue_idx: Pos2ClueIdx,
    pub cur_pos: Pos, // row idx, col idx
    pub cur_dir: TypingDir,
    pub cur_sel: SelectedWord,// BTreeSet<Pos>,
    pub title: String,
}

impl Board {
    pub fn new(
        title: String, 
        content: Vec<Vec<Cell>>,
        solution: Vec<Vec<Cell>>,  
        clues: Vec<String>, 
        pos_2_clue_idx: Pos2ClueIdx) -> Self {
        let mut ret = Board {
            content,
            solution,
            title,
            cur_pos: Pos::new(0, 0),
            cur_dir: TypingDir::Across,
            cur_sel: SelectedWord::default(),
            clues,
            pos_2_clue_idx: pos_2_clue_idx,
        };
        
        ret.update_selection();
        ret
    }
    
    pub fn height(&self) -> usize {
        self.content.len()
    }

    pub fn width(&self) -> usize {
        if !self.content.is_empty() {
            self.content[0].len()
        } else {
            panic!("board not initialized?!")
        }
    }
    
    pub fn is_black_cell(&self, pos: &Pos) -> bool {
        self.solution_tile_at(pos) == Some(Cell::Black)
        
    }

    fn move_cursor(&mut self, dir: Direction) -> bool {
        let height  = self.height();
        let width = self.width();

        if height == 0 || width == 0 {
            panic!("board not initialized?!")
        }

        let mut result = self.cur_pos;

        let wrapped: bool = match dir {
            Direction::Up => {
                result.row = if result.row >= 1 {
                    result.row - 1
                } else {
                    height - 1
                };
                result.row == height - 1 
            }
            Direction::Down => {
                result.row = (result.row + 1) % height;
                result.row == 0
             }
            Direction::Left => {
                result.col = if result.col >= 1 {
                    result.col - 1
                } else {
                    width - 1
                };
                result.col == width - 1
            }
            Direction::Right => {
                result.col = (result.col + 1) % width;
                result.col == 0
            },
        };

        // Nprintln!("cur_pos={cp:?} dir={dir:?} new_pos={result:?}", cp=self.cur_pos);
        self.cur_pos = result;
        self.update_selection();
        wrapped
    }

    pub fn type_letter(&mut self, ch: char, move_next: bool) {
        let ri = self.cur_pos.row;
        let ci = self.cur_pos.col;
        self.content[ri][ci] = if ch != ' '  {
            let right = self.solution[ri][ci].get_char();

            if ch == right {
                Cell::OccupiedRight(ch)
            } else {
                Cell::OccupiedWrong(ch)
            }
        } else {
            Cell::Empty
        };
        
        let dir = match self.cur_dir {
            TypingDir::Across => Direction::Right,
            TypingDir::Down => Direction::Down
        };
        if move_next {
            self.move_cursor_until_not_black(dir)
        }
    }

    fn solution_tile_at(&self, pos: &Pos) -> Option<Cell> {
        if pos.row < self.height() && pos.col < self.width() {
            Some(self.solution[pos.row][pos.col])
        } else {
            None
        }
    }
    
    pub fn toggle_typing_dir(&mut self) { 
        self.cur_dir = self.cur_dir.toggle();
        self.update_selection()
    }

    fn update_selection(&mut self) {
        self.cur_sel = self.recalc_selection();
    }

    fn recalc_selection(&self) -> SelectedWord {
        let mut selection = BTreeSet::new();

        let (delta_up, delta_down) = if self.cur_dir == TypingDir::Across {
            (DeltaPos::new(0, 1), DeltaPos::new(0, -1))
        } else {
            (DeltaPos::new(-1, 0), DeltaPos::new(1, 0))
        };

        self.extend_selection_dir(&mut selection, &delta_up);
        self.extend_selection_dir(&mut selection, &delta_down);

        SelectedWord{poss: selection}
    }

    fn extend_selection_dir(&self, selection: &mut BTreeSet<Pos>, delta: &DeltaPos) {
        let mut pos =  self.cur_pos.clone();

        while let Some(tile) = self.solution_tile_at(&pos) {
            if tile == Cell::Black {
                break
            }
            selection.insert(pos.clone());
            pos.add_ip(delta);
        }
    }
    
    pub fn move_cursor_until_not_black(&mut self, dir: Direction) {
        let  alter_dir = dir.alternate();
        loop {
            let wrapped = self.move_cursor(dir);
            if wrapped {
                self.move_cursor(alter_dir);
            }
            if !self.is_black_cell(&self.cur_pos) {
                break
            }
        }
    }
    
    pub fn current_clue(&self) -> String {
        let first_pos_opt = self.cur_sel.first_pos();
        if let Some(first_pos) = first_pos_opt {
            if let Some(clue_idx) = self.pos_2_clue_idx.0.get(&(first_pos, self.cur_dir)) { 
                format!("{first_pos:?} {d:?} c_idx={ci}\n{clue}", d=self.cur_dir, 
                    ci=*clue_idx, clue=self.clues[*clue_idx])
            } else {
                format!("pos: {first_pos:?}, could not find clue_idx!")
            }
        } else {
            format!("No first pos for current selection? cur_sel={:?}", self.cur_sel)
        }
    }
}



