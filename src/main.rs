use iced::Application;
use iced::{self, Settings};

pub mod state;
pub mod app;
pub mod view;
pub mod api_types;
pub mod import_puz;


use state::Board;


fn main() -> iced::Result {

    // let header = read_header(&mut f).unwrap();

    Board::run(Settings::default())
}
