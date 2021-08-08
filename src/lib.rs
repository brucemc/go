
use thiserror::Error;

mod parser;
use self::parser::Parser;
use self::parser::SgfNode;

mod board;
use self::board::Board;

mod game;
pub use self::game::Game;


#[derive(Error, Debug)]
pub enum GoError {
    #[error("file not found")]
    FileNotFound{source: std::io::Error},
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("invalid move: {0}")]
    InvalidMove(String),
    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("other error: {0}")]
    Other(String),

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Black,
    White,
}

impl std::string::ToString for Color {
    fn to_string(&self) -> String {
        match self {
            Color::Black => "black".to_string(),
            Color::White => "white".to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct Intersection {
    row: usize,
    col: usize,
}

// The first letter designates the column (left to right), the second the row (top to bottom).
// The upper left part of the board is used for smaller boards, e.g. letters "a"-"m" for 13*13.
// The author intentionally broke with the tradition of labeling moves (and points) with letters
// "A"-"T" (excluding "i") and numbers 1-19. Two lower-case letters in the range "a"-"s" were
// used instead, for reasons of simplicity and compactness.
//
//     0   1   2   3       17  18
//   0 aa  ba  ca  da  ... ra  sa
//   1 ab  bb  cb  db  ... rb  sb
//   2 ac  bc  cc  dc  ... rc  sc
//     ...
//  17 ar  br  cr  dr  ... rr  sr
//  18 as  bs  cs  ds  ... rs  ss
//
//     0   1   2   3       17  18
//   0 A19 B19 C19 D19 ... S19 T19
//   1 A18 B18 C18 D18 ... S18 T18
//   2 A17 B17 C17 D17 ... S17 T17
//     ...
//  17 A2  B2  C2  D2  ... S2  T2
//  18 A1  B1  C1  D1  ... S1  T1

impl Intersection {
    fn new(row: usize, col: usize) -> Intersection {
        Intersection { row, col }
    }

    fn from_sgf_coord(sgf_coord: &[char]) -> Intersection {
        Intersection {
            col: sgf_coord[0] as usize - 'a' as usize,
            row: sgf_coord[1] as usize - 'a' as usize,

        }
    }

    fn up(&self) -> Intersection {
        Intersection {
            row: self.row + 1,
            col: self.col,
        }
    }

    fn down(&self) -> Intersection {
        Intersection {
            row: self.row - 1,
            col: self.col,
        }
    }

    fn left(&self) -> Intersection {
        Intersection {
            row: self.row,
            col: self.col - 1,
        }
    }

    fn right(&self) -> Intersection {
        Intersection {
            row: self.row,
            col: self.col + 1,
        }
    }

    fn to_coord(&self) -> String {
        let mut ret: String = match self.col {
            0..=7 => (self.col as u8 + b'a') as char,
            _ => (self.col as u8 + b'b') as char,
        }
            .to_string();
        ret += &(self.row + 1).to_string();
        return ret;
    }
}

#[derive(Debug, Clone)]
pub enum PointState {
    Empty,
    Filled {
        move_number: usize,
        stone_color: Color,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Move {
    move_number: usize,
    intersection: Intersection,
    color: Color,
}

#[cfg(test)]
mod tests {
    use crate::game::Game;
    use crate::Intersection;
    use crate::Color;

    #[test]
    fn move_numbers() {
        let game = Game::new(19);
        assert_eq!(game.get_move_number(), 0);
        let game = Game::from_sgf_file("./resources/The_59th_Judan_Title_Match_3rd_game.sgf".to_string()).unwrap();
        assert_eq!(game.get_move_number(), 337);
    }

    #[test]
    fn place_stones() {
        let mut game = Game::new(19);
        game.place_handicap_stone(Intersection::new(2,2)).ok();
        assert_eq!(game.get_move_number(), 0);
        game.place_stone(Intersection::new(3,3), Color::White).ok();
        assert_eq!(game.get_move_number(), 1);
        let board = game.get_board(1).unwrap();
        assert_eq!(board.to_ascii(),
            ".  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  X  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  O  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n\
             .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  .  \n");
    }
}

