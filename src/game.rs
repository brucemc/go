use sgf_parser::*;
use std::fs;

use anyhow::{Result};
use std::array::IntoIter;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::iter::FromIterator;

use super::Board;
use super::GoError;
use super::Intersection;
use super::Move;

#[derive(Default, Debug, Clone)]
pub struct Game {
    board_size: u32,
    player_black: String,
    player_white: String,
    rank_black: String,
    rank_white: String,
    board_positions: BTreeMap<usize, Board>,
    handicap_stones: HashSet<Intersection>,
    move_number: usize,
    moves: BTreeMap<usize, Move>,
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
            handicap_stones: HashSet::new(),
            move_number: 0,
            moves: BTreeMap::new(),
        }
    }

    pub fn from_game_tree(tree: &sgf_parser::GameTree) -> Game {
        let mut game = Game::default();
        let mut iter = tree.iter();

        iter.for_each(|node| {
            let mut node_iter = node.tokens.iter();
            node_iter.for_each(|token|{
                match &token {
                    SgfToken::Size (size,_) => {
                        game.board_size = *size;
                        game.board_positions = BTreeMap::from_iter(IntoIter::new([(0, Board::new(game.board_size))]));
                    },
                    SgfToken::PlayerName {color: sgf_parser::Color::Black, name} => {
                        game.player_black = name.to_string();
                    },
                    SgfToken::PlayerName {color: sgf_parser::Color::White, name} => {
                        game.player_white = name.to_string();
                    },
                    SgfToken::PlayerRank {color: sgf_parser::Color::Black, rank} => {
                        game.rank_black = rank.to_string();
                    },
                    SgfToken::PlayerRank {color: sgf_parser::Color::White, rank} => {
                        game.rank_white = rank.to_string();
                    },
                    SgfToken::Add {color: sgf_parser::Color::Black, coordinate} => {
                        game.place_handicap_stone(Intersection::from_sgf(coordinate.1.into(), coordinate.0.into()));
                    },
                    SgfToken::Move {color, action: Action::Move(col,row)} => {
                        game.place_stone(Intersection::from_sgf(*row as u32, *col as u32), *color);
                    },
                    _ => {}
                }
            });
        });
        return game;
    }

    pub fn get_board_size(&self) -> u32 {
        self.board_size
    }

    pub fn get_final_move_number(&self) -> usize {
        self.move_number
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

    pub fn get_board(&self, move_number: usize) -> Result<Board, GoError> {
        if let Some(board) = self.board_positions.get(&move_number) {
           Ok(board.clone())
        }
        else {
            Err(GoError::InvalidMove(
                "board not found".to_string()
            ))
        }
    }

    pub fn place_stone(&mut self, intersection: Intersection, color: Color) -> Result<(), GoError> {
        if let Some(board) = self.board_positions.get(&self.move_number) {
            let mut board = board.clone();
            self.move_number += 1;
            board.place_stone(
                Some(self.move_number),
                intersection,
                color,
            )?;
            self.board_positions.insert(self.move_number, board);
            self.moves.insert(self.move_number, Move{move_number: self.move_number, intersection, color});
            Ok(())
        } else {
            Err(GoError::InvalidMove(
                "No previous board position".to_string(),
            ))
        }
    }

    pub fn place_handicap_stone(&mut self, intersection: Intersection) -> Result<(), GoError> {
        if let Some(board) = self.board_positions.get_mut(&0) {
            board.place_stone(None, intersection, Color::Black)?;
            self.handicap_stones.insert(intersection);
            Ok(())
        } else {
            Err(GoError::InvalidMove("Invalid board".to_string()))
        }
    }

    pub fn from_sgf_file(file_name: String) -> Result<Game, GoError> {
        let sgf_source = fs::read_to_string(file_name)?;
        let tree= parse(sgf_source.as_str())?;

        let mut game = Game::from_game_tree(&tree);


        // for sgf_node in &sgf_nodes {
        //     match sgf_node {
        //         SgfNode::PlaceHandicapStone(intersection) => {
        //             game.place_handicap_stone(*intersection)?;
        //         }
        //         SgfNode::PlaceStone(intersection, color) => {
        //             game.place_stone(
        //                 *intersection,
        //                 *color)?;
        //
        //         }
        //     }
        // }
        return Ok(game);
    }

    pub fn render_to_latex(&self, step_size: usize) -> Result<String, GoError> {
        let mut move_number = 0;
        let mut ret: String = "".to_string();
        while move_number < self.move_number {
            ret += &self.render_board_to_latex(
                if move_number + step_size < self.move_number {
                    move_number + step_size
                } else {
                    self.move_number
                },
                Some(move_number),
            )?;
            move_number += step_size;
        }
        Ok(ret)
    }

    pub fn render_board_to_latex(
        &self,
        move_number: usize,
        number_from: Option<usize>,
    ) -> Result<String, GoError> {
        let board = self
            .board_positions
            .get(&move_number)
            .ok_or(GoError::InvalidMove(move_number.to_string()))?;
        board.render_diagram(number_from, &self.moves)
    }
}
