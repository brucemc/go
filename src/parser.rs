use anyhow::{Result};
use utf8_chars::BufReadCharsExt;

use super::Color;
use super::Intersection;
use crate::GoError;

pub struct Parser {
    //file_name: String,
    buffer_reader: std::io::BufReader<std::fs::File>,
    current_char: Option<char>,
    board_size: usize,
    player_black: String,
    player_white: String,
    rank_black: String,
    rank_white: String,
}

#[derive(Debug, Clone)]
pub enum SgfNode {
    PlaceStone(Intersection, Color),
    PlaceHandicapStone(Intersection),
}

impl Parser {
    pub fn new(file_name: String) -> Result<Parser, GoError> {
        let f = std::fs::File::open(&file_name).map_err(|source| GoError::FileNotFound{source})?;
        Ok(Parser {
            //file_name,
            buffer_reader: std::io::BufReader::new(f),
            current_char: None,
            board_size: 19, // default
            player_black: "".to_string(),
            player_white: "".to_string(),
            rank_black: "".to_string(),
            rank_white: "".to_string(),
        })
    }

    pub fn get_board_size(&self) -> usize {
        self.board_size
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

    pub fn parse(&mut self) -> Result<Vec<SgfNode>, GoError> {
        let mut ret: Vec<SgfNode> = vec![];

        self.getch();

        self.expect('(')?;
        self.skip_ws();

        while self.current_char == Some(';') {
            self.getch();
            self.skip_ws();

            while self.current_char != None && self.current_char.unwrap().is_alphabetic() {
                let command = self.get_command();
                self.skip_ws();

                while self.current_char == Some('[') {
                    self.expect('[')?;
                    let mut sgf_node_value: String = "".to_string();
                    while self.current_char != None && self.current_char != Some(']') {
                        sgf_node_value.push(self.current_char.unwrap());
                        self.getch();
                    }
                    self.expect(']')?;
                    self.skip_ws();

                    match command.as_str() {
                        "PB" => {
                            self.player_black = sgf_node_value.chars().collect();
                        }
                        "BR" => {
                            self.rank_black = sgf_node_value.chars().collect();
                        }
                        "PW" => {
                            self.player_white = sgf_node_value.chars().collect();
                        }
                        "WR" => {
                            self.rank_white = sgf_node_value.chars().collect();
                        }
                        "SZ" => {
                            let sgf_size: String = sgf_node_value.chars().collect();
                            self.board_size = sgf_size.parse::<usize>().unwrap();
                        }
                        "W" => {
                            let sgf_coord: Vec<char> = sgf_node_value.chars().collect();
                            ret.push(SgfNode::PlaceStone(Intersection::from_sgf_coord(&sgf_coord),Color::White));
                        }
                        "B" => {
                            let sgf_coord: Vec<char> = sgf_node_value.chars().collect();
                            ret.push(SgfNode::PlaceStone(Intersection::from_sgf_coord(&sgf_coord), Color::Black));
                        }
                        "AB" => {
                            let sgf_coord: Vec<char> = sgf_node_value.chars().collect();
                            ret.push(SgfNode::PlaceHandicapStone(Intersection::from_sgf_coord(
                                &sgf_coord,
                            )));
                        }
                        _ => {}
                    }
                }
            }
        }

        self.expect(')')?;
        Ok(ret)
    }

    fn get_command(&mut self) -> String {
        self.skip_ws();
        let mut s: String = "".to_string();
        while let Some(c) = self.current_char {
            if c.is_alphabetic() {
                s.push(c);
                self.getch();
            } else {
                return s;
            }
        }
        s
    }

    fn skip_ws(&mut self) {
        while self.current_char != None {
            match self.current_char {
                Some(' ') => {
                    self.getch();
                }
                Some('\r') => {
                    self.getch();
                }
                Some('\n') => {
                    self.getch();
                }
                _ => {
                    return;
                }
            }
        }
    }

    fn expect(&mut self, c: char) -> Result< (), GoError> {
        self.skip_ws();
        match self.current_char {
            Some(current_char) => {
                if current_char != c {
                    return Err(GoError::ParseError(format!(
                        "Unexpected character: expected {:?} got {:?}",
                        c, self.current_char
                    )));
                }
                self.getch();
            }
            None => {
                return Err(GoError::ParseError(format!(
                    "Unexpected character: expected {:?} got {:?}",
                    c, self.current_char
                )));
            }
        }
        Ok(())
    }

    fn getch(&mut self) {
        self.current_char = self.buffer_reader.chars().next().map_or(None, |x| match x {
            Ok(x) => Some(x),
            Err(_) => None,
        });
    }
}
