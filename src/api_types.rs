
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn alternate(&self) -> Self {
        match self {
            Self::Right => Self::Down,
            Self::Down => Self::Right,
            Self::Up => Self::Left,
            Self::Left => Self::Up,
        }
    }
}


#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum TypingDir {
    Across,
    Down,
}

impl TypingDir {
    pub fn toggle(&self) -> Self {
        match self {
            Self::Across => Self::Down,
            Self::Down => Self::Across
        }
    }
}

impl Into<Direction> for TypingDir {
    fn into(self) -> Direction {
        match self {
            Self::Across => Direction::Right,
            Self::Down => Direction::Down
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub row: usize,
    pub col: usize
}


impl Pos {
    pub fn new(row: usize, col: usize) -> Self {
        Pos{row, col}
    }

    pub fn add_ip(&mut self, delta: &DeltaPos) -> &Self {
        self.row = (self.row as i32 + delta.row) as usize;
        self.col = (self.col as i32 + delta.col) as usize;

        self
    }
}

pub struct DeltaPos {
    row: i32,
    col: i32
}

impl DeltaPos {
    pub fn new(row: i32, col: i32) -> Self {
        DeltaPos{row, col}
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Msg {
    MoveCursor(Direction),
    TypeLetter(char),
    ClearCell,
    ToggleTypingDir,
}