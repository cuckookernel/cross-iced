use std::fs::File;
use std::io::BufReader;

use iced::alignment::Horizontal;
use iced::keyboard::key::Named;
use iced::keyboard::{on_key_release, Key, Modifiers};

use iced::widget::vertical_space;
use iced::Border;
use iced::{
    executor,
    widget::{container, text, Column, Row, row, column},
    Application, Color, Command, Element, Theme,
};

use crate::import_puz;
use crate::import_puz::ImportedPuz;
use crate::state::*;
use crate::api_types::{Pos, Msg, Direction};

const TEXT_SIZE: u16 = 30;
const CELL_HEIGHT: f32 = 40.;
const CELL_WIDTH: f32 = 40.;

type CellStyler = fn(&Theme) -> container::Appearance;

impl Application for Board {
    type Executor = executor::Default;
    type Message = Msg;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Msg>) {
        // let content = vec![vec![Tile::Empty; 16]; 16];
        let imported_puz = load_test_puz();
        
        let solution = solution_from(&imported_puz);
        let content = solution.clone();

        let board = Board::new(
            "Cross-Iced".to_string(),
            content,
            solution,
            imported_puz.clues().clone(),
            imported_puz.pos_2_clue_idx
            );
        
        (board, Command::none())

    
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn view(&self) -> Element<Msg> {
        row![
            text(" "),
            column![
            vertical_space(),
            row![self.draw_board(), self.draw_clue_pane()].spacing(10),
            vertical_space()
            ]
        ].into()
    }
    
    fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg::MoveCursor(dir) => self.move_cursor_until_not_black(dir),
            Msg::TypeLetter(ch) => self.type_letter(ch, true),
            Msg::ClearCell => self.type_letter(' ', false),
            Msg::ToggleTypingDir => self.toggle_typing_dir() 
        }

        Command::none()
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        on_key_release::<Msg>(|k, m| {
            println!("On key release: k={k:?} mod={m:?}");

            match k {
                Key::Named(Named::ArrowLeft) => Some(Msg::MoveCursor(Direction::Left)),
                Key::Named(Named::ArrowRight) => {
                    if m.contains(Modifiers::SHIFT) || m.contains(Modifiers::CTRL) {    
                        Some(Msg::ToggleTypingDir)
                    } else {
                        Some(Msg::MoveCursor(Direction::Right))
                    }
                },
                Key::Named(Named::ArrowUp) => Some(Msg::MoveCursor(Direction::Up)),
                Key::Named(Named::ArrowDown) => {
                    if m.contains(Modifiers::SHIFT) || m.contains(Modifiers::CTRL) {
                        Some(Msg::ToggleTypingDir)
                    } else {
                        Some(Msg::MoveCursor(Direction::Down))    
                    }
                },
                Key::Named(Named::Backspace) => Some(Msg::ClearCell),
                Key::Named(Named::Space) => Some(Msg::ClearCell),
                Key::Character(st) => {
                    let c_str = st.to_string();
                    if c_str.len() > 0 {
                        Some(Msg::TypeLetter(c_str.chars().last().unwrap().to_ascii_uppercase()))
                    } else {
                        None
                    }
                }
                _ => None
            }
        })
    }
}


impl Board {
    fn draw_clue_pane(&self) -> Element<Msg> {
        text(self.current_clue()).size(20).into()
    }
    
    fn draw_board(&self) -> Element<Msg> {
        let mut col_children: Vec<Element<Msg>> = Vec::new();
        for (r_idx, content_row) in self.content.iter().enumerate() {
            let mut row: Vec<Element<Msg>> = Vec::new();
            for (c_idx, tile) in content_row.iter().enumerate() {
                let pos = Pos::new(r_idx, c_idx);
                let styler : CellStyler = if pos == self.cur_pos {
                        active_cell
                    } else if self.cur_sel.contains(&pos) {
                        selected_cell
                    } else {
                        inactive_cell
                    };

                let (text_elem, styler) : (Element<_, _>, CellStyler) =
                    match tile {
                        Cell::Empty => (cell(' ', true), styler),
                        Cell::Black => (cell(' ', true), black_block),
                        Cell::OccupiedRight(c) => (cell(*c, true), styler),
                        Cell::OccupiedWrong(c) => (cell(*c, false), styler)
                    }
                ;

                let cont = container(text_elem)
                    .height(CELL_HEIGHT)
                    .width(CELL_WIDTH)
                    .style(styler);
                row.push(cont.into())
            }
            let row = Row::from_vec(row).spacing(0);
            col_children.push(row.into())
        }

        Column::from_vec(col_children).spacing(0).into()
    }
}

fn load_test_puz() -> ImportedPuz {
    let path = "/home/teo/Downloads/wsj240702.puz";
    let mut f = BufReader::new(File::open(path).unwrap());

    // let header = read_header(&mut f).unwrap();
    import_puz::import_puzzle(&mut f).unwrap()

}
    
fn solution_from(imported_puz: &ImportedPuz) -> Vec<Vec<Cell>> {
    let mut ret = Vec::new();

    for r_idx in 0..imported_puz.height() {
        let row_ret = (0..imported_puz.width()).map (
            |c_idx| {
                match imported_puz.solution_at(r_idx, c_idx) {
                    '.' => Cell::Black,
                    '_' => Cell::Empty,
                    c => Cell::OccupiedRight(c)
                }
        });
        ret.push(row_ret.collect())
    }

    ret
}

fn active_cell(_th: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb8(  150, 250, 255))),
        ..default_cell_appearance(_th)
    }
}

fn selected_cell(_th: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb8(  200, 250, 255))),
        /* border: Border{
            color: Color::from_rgb8(  150, 250, 255),
            radius: 0.into(),
            width: 1.0,

        },*/
        ..default_cell_appearance(_th)
    }
}

fn inactive_cell(_th: &Theme) -> container::Appearance {
    container::Appearance {
        ..default_cell_appearance(_th)
    }
}

fn default_cell_appearance(_th: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb8(  255, 255, 255))),
        border: Border{
            color: Color::BLACK,
            radius: 0.into(),
            width: 1.0,

        },
        ..Default::default()
    }
}

fn black_block(_th: &Theme) -> container::Appearance {
    container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgb8(10, 10, 10))),
        ..Default::default()
    }
}


fn cell<'a>(ch: char, right: bool) -> Element<'a, Msg, Theme> {
    let content = ch.to_string();
    text(content)
        .size(TEXT_SIZE)
        .height(CELL_HEIGHT)
        .width(CELL_WIDTH)
        .style(if right {Color::BLACK} else {Color::from_rgb8(255, 0, 0)})
        .horizontal_alignment(Horizontal::Center)
        .into()
}

