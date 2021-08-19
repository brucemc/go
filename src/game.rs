use sgf_parser::*;
use std::fs;

use anyhow::{Result};
use std::array::IntoIter;
use std::collections::BTreeMap;
use std::iter::FromIterator;

use super::Board;
use super::Error;
use super::Intersection;

#[derive(Default, Debug, Clone)]
pub struct Game {
    board_size: u32,
    player_black: String,
    player_white: String,
    rank_black: String,
    rank_white: String,
    board_positions: BTreeMap<u32, Board>,
    board_number: u32,
}

impl Game {
    pub fn new(board_size: u32) -> Game {
        Game {
            board_size,
            player_black: "".to_string(),
            player_white: "".to_string(),
            rank_black: "".to_string(),
            rank_white: "".to_string(),
            board_positions: BTreeMap::from_iter(IntoIter::new([(0, Board::new(board_size))])),
            board_number: 0,
        }
    }

    pub fn from_game_tree(tree: &sgf_parser::GameTree) -> Result<Game, Error> {
        let mut game = Game::default();

        for game_node in &tree.nodes {
            for token in &game_node.tokens {
                match &token {
                    SgfToken::Size(size, _) => {
                        game.board_size = *size;
                        game.board_positions = BTreeMap::from_iter(IntoIter::new([(0, Board::new(game.board_size))]));
                    },
                    SgfToken::PlayerName { color: sgf_parser::Color::Black, name } => {
                        game.player_black = name.to_string();
                    },
                    SgfToken::PlayerName { color: sgf_parser::Color::White, name } => {
                        game.player_white = name.to_string();
                    },
                    SgfToken::PlayerRank { color: sgf_parser::Color::Black, rank } => {
                        game.rank_black = rank.to_string();
                    },
                    SgfToken::PlayerRank { color: sgf_parser::Color::White, rank } => {
                        game.rank_white = rank.to_string();
                    },
                    _ => {}
                }
            }
        }

        game.add_moves(tree, 0)?;
        Ok(game)
    }

    fn add_moves(&mut self, tree: &sgf_parser::GameTree, board_number: u32 ) -> Result<(), Error> {

        let mut bn = board_number;

        for game_node in &tree.nodes {
            for token in &game_node.tokens {
                match &token {
                    SgfToken::Add { color: sgf_parser::Color::Black, coordinate } => {
                        self.place_handicap_stone(Intersection::from_sgf(coordinate.1.into(), coordinate.0.into()))?;
                    },
                    SgfToken::Move { color, action: Action::Move(col, row) } => {
                        self.place_stone(Intersection::from_sgf(*row as u32, *col as u32), *color, bn)?;
                        bn = self.board_number;
                    },
                    _ => {}
                }
            }
        }
        let bn = self.board_number;
        for v in &tree.variations {
            self.add_moves(&v, bn)?;
        }
        Ok(())
    }

    pub fn get_board_size(&self) -> u32 {
        self.board_size
    }

    pub fn get_final_move_number(&self) -> u32 {
        self.board_number
    }

    pub fn get_player_black(&self) -> String {
        self.player_black.clone()
    }

    pub fn get_player_white(&self) -> String {
        self.player_white.clone()
    }

    pub fn get_rank_black(&self) -> String {
        self.rank_black.clone()
    }

    pub fn get_rank_white(&self) -> String {
        self.rank_white.clone()
    }

    pub fn get_board(&self, board_number: u32) -> Result<Board, Error> {
        if let Some(board) = self.board_positions.get(&board_number) {
           Ok(board.clone())
        }
        else {
            Err(Error::InvalidBoardNumber(
                "board not found".to_string()
            ))
        }
    }

    pub fn place_stone(&mut self, intersection: Intersection, color: Color, board_number : u32) -> Result<(), Error> {
        if let Some(board) = self.board_positions.get_mut(&board_number) {
            self.board_number += 1;
            let mut new_board = board.clone();
            new_board.clear_next();
            board.add_next(self.board_number);
            new_board.set_prev(board_number);
            new_board.place_stone(
                intersection,
                color,
            )?;
            self.board_positions.insert(self.board_number, new_board);
//            self.moves.insert(self.board_number, Move{move_number: self.board_number, intersection, color});
            Ok(())
        } else {
            Err(Error::InvalidBoardNumber(
                "No previous board position".to_string(),
            ))
        }
    }

    pub fn place_handicap_stone(&mut self, intersection: Intersection) -> Result<(), Error> {
        if let Some(board) = self.board_positions.get_mut(&0) {
            board.add_stone(intersection, Color::Black)?;
//            self.handicap_stones.insert(intersection);
            Ok(())
        } else {
            Err(Error::InvalidBoardNumber("Invalid board".to_string()))
        }
    }

    pub fn from_sgf_file(file_name: String) -> Result<Game, Error> {
        let sgf_source = fs::read_to_string(file_name)?;
        let tree= parse(sgf_source.as_str())?;
        Game::from_game_tree(&tree)
    }

    pub fn render_to_latex(&self, step_size: u32) -> Result<String, Error> {
        let mut move_number = 0;
        let mut ret: String = "".to_string();
        while move_number < self.board_number {
            ret += &self.render_board_to_latex(
                if move_number + step_size < self.board_number {
                    move_number + step_size
                } else {
                    self.board_number
                },
                Some(move_number),
            )?;
            move_number += step_size;
        }
        Ok(ret)
    }

    pub fn render_board_to_latex(
        &self,
        move_number: u32,
        number_from: Option<u32>,
    ) -> Result<String, Error> {
        let board = self
            .board_positions
            .get(&move_number)
            .ok_or(Error::InvalidBoardNumber(move_number.to_string()))?;
        board.render_diagram(number_from)
    }
}
