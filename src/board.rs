use sgf_parser::*;
use alphabet::*;
use anyhow::Result;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;

use super::color_to_string;
use super::Error;
use super::Intersection;
use super::Move;
use super::PointState;

#[derive(Debug, Clone)]
enum GroupPoint {
    Ungrouped,
    Grouped { group_number: i32 },
}

#[derive(Debug, Clone)]
pub struct Board {
    size: u32,
    points: array2d::Array2D<PointState>,
    prev_board: u32,
    next_board: Vec<u32>,
    move_number: u32,
    moves: BTreeMap<u32, Move>,
}

impl Board {
    pub fn new(size: u32) -> Board {
        Board {
            size,
            points: array2d::Array2D::filled_with(PointState::Empty, size as usize, size as usize),
            prev_board: 0,
            next_board: vec![],
            move_number: 0,
            moves: BTreeMap::new(),
        }
    }

    pub fn get_size(&self) -> u32 { self.size }

    pub fn set_prev(&mut self, board_number: u32) {
        self.prev_board = board_number;
    }

    pub fn get_prev(&self) -> u32 {
        self.prev_board
    }

    pub fn clear_next(&mut self) {
        self.next_board.clear();
    }

    pub fn add_next(&mut self, board_number: u32) {
        self.next_board.push(board_number);
    }

    pub fn get_next(&self, variation: u32) -> Option<u32> {
        if self.next_board.len() > variation as usize {
            Some(self.next_board[variation as usize])
        }
        else {
            None
        }
    }

    pub fn get_next_boards(&self) -> Vec<u32> {
        self.next_board.clone()
    }

    pub fn get_last_move(&self) -> Move {
        self.moves[&self.move_number].clone()
    }

    pub fn get_variation_count(&self) -> u32 {
        self.next_board.len() as u32
    }

    pub fn get_point(&self, r: u32, c: u32) -> Result<PointState, Error> {
        if let Some(p) = self.points.get(r as usize, c as usize).clone() {
            Ok(p.clone())
        } else {
            Err(Error::InvalidBoardNumber("No Point".to_string()))
        }
    }

    pub fn render_diagram(
        &self,
        from_move: Option<u32>,
    ) -> Result<String, Error> {
        let mut ret: String = "".to_string();
        let mut cap_ret: String = "".to_string();
        let mut max_move: u32 = 0;
        let mut numbered_moves: HashSet<u32> = HashSet::new();
        let mut captured_moves: BTreeMap<Intersection, BTreeSet<u32>> = BTreeMap::new();

        for c in 0..self.size {
            for r in 0..self.size {
                match self.points.get(r as usize, c as usize) {
                    Some(&PointState::Filled {
                        move_number: 0,
                        stone_color: _,
                    }) => {
                        ret += &r#"\black{"#.to_string();
                        ret += &Intersection::new(r, c).to_coord();
                        ret += &"}\n";
                    }

                    Some(&PointState::Filled {
                        move_number,
                        stone_color,
                    }) => {
                        ret += &r#"\"#.to_string();
                        ret += &color_to_string(&stone_color);
                        if move_number >= from_move.unwrap_or(0) {
                            ret += &"[";
                            ret += &move_number.to_string();
                            ret += &"]";
                            numbered_moves.insert(move_number);
                        }
                        ret += &"{";
                        ret += &Intersection::new(r, c).to_coord();
                        ret += &"}\n";
                        if move_number > max_move {
                            max_move = move_number;
                        }
                    }
                    _ => {}
                }
            }
        }

        for move_num in from_move.unwrap_or(0)..max_move {
            if move_num > 0 && !numbered_moves.contains(&move_num) {
                if let Some(m) = &self.moves.get(&move_num) {
                    captured_moves
                        .entry(m.intersection)
                        .or_insert_with(BTreeSet::new)
                        .insert(move_num);
                }
            }
        }

        alphabet!(CAPS = "ABCDEFGHIJKLMNOPQRST");
        let mut caps = CAPS.iter_words_counting();
        for (intersection, move_list) in &captured_moves {
            cap_ret += &move_list.iter().join(", ");
            cap_ret += &" at ";
            if let Some(&PointState::Filled {
                move_number,
                stone_color: _,
            }) = self.points.get(intersection.row as usize, intersection.col as usize)
            {
                cap_ret += &move_number.to_string();
            } else {
                let loc = caps.next().unwrap_or("Z".to_string());
                cap_ret += &loc;

                ret += &r#"\gobansymbol{"#.to_string();
                ret += &intersection.to_coord();
                ret += &"}{";
                ret += &loc;
                ret += &"}\n";
            };
            cap_ret += &"\\\\\n";
        }

        ret += &"\n";
        ret += "\\begin{center}\n";
        ret += "\\rotategobanright\n";
        ret += "\\shortstack{\\showfullgoban \\\\ From move ";
        ret += &from_move.unwrap_or(0).to_string();
        ret += "}\n";
        ret += "\\end{center}\n";
        ret += "\\cleargoban\n";
        ret += &cap_ret;
        Ok(ret)
    }

    pub fn add_stone(
        &mut self,
        intersection: Intersection,
        stone_color: Color,
    ) -> Result<(), Error> {
        match self.points.get(intersection.row as usize, intersection.col as usize) {
            Some(&PointState::Empty) => {
                self.points
                    .set(
                        intersection.row as usize,
                        intersection.col as usize,
                        PointState::Filled {
                            move_number: 0,
                            stone_color,
                        },
                    )
                    .ok();
                self.remove_captures(stone_color).ok();
                Ok(())
            }
            Some(&PointState::Filled {
                move_number: _,
                stone_color: _,
            }) => Err(Error::InvalidMove("Point already filled".to_string())),
            None => Err(Error::InvalidMove("Point not found".to_string())),
        }
    }

    pub fn place_stone(
        &mut self,
        intersection: Intersection,
        stone_color: Color,
    ) -> Result<(), Error> {
        match self.points.get(intersection.row as usize, intersection.col as usize) {
            Some(&PointState::Empty) => {
                self.move_number += 1;
                self.moves.insert(self.move_number, Move { move_number: self.move_number, intersection, color: stone_color });
                self.points
                    .set(
                        intersection.row as usize,
                        intersection.col as usize,
                        PointState::Filled {
                            move_number: self.move_number,
                            stone_color,
                        },
                    )
                    .ok();
                self.remove_captures(stone_color).ok();
                Ok(())
            }
            Some(&PointState::Filled {
                move_number: _,
                stone_color: _,
            }) => Err(Error::InvalidMove("Point already filled".to_string())),
            None => Err(Error::InvalidMove("Point not found".to_string())),
        }
    }

    fn remove_captures(&mut self, placed_stone_color: Color) -> Result<(), String> {
        let mut group_assignments: array2d::Array2D<GroupPoint> =
            array2d::Array2D::filled_with(GroupPoint::Ungrouped, self.size as usize, self.size as usize);
        let mut group_liberties: HashSet<Intersection> = HashSet::new();
        let mut group_members: HashSet<Intersection> = HashSet::new();
        let mut group_number = 1;
        for r in 0..self.size {
            for c in 0..self.size {
                match self.points.get(r as usize, c as usize) {
                    Some(&PointState::Filled {
                        move_number: _,
                        stone_color,
                    }) => {
                        if let Ok(result) = self.check_point(
                            &mut group_assignments,
                            &mut group_liberties,
                            &mut group_members,
                            stone_color,
                            Intersection::new(r, c),
                            group_number,
                        ) {
                            if result {
                                if group_liberties.len() == 0 && placed_stone_color != stone_color {
                                    for intersection in &group_members {
                                        self.points
                                            .set(
                                                intersection.row as usize,
                                                intersection.col as usize,
                                                PointState::Empty,
                                            )
                                            .or(Err(Error::InvalidBoardNumber(
                                                "Could not set point".to_string(),
                                            )))
                                            .ok();
                                    }
                                }
                                group_number += 1;
                                group_liberties.clear();
                                group_members.clear();
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn check_point(
        &mut self,
        group_assignments: &mut array2d::Array2D<GroupPoint>,
        group_liberties: &mut HashSet<Intersection>,
        group_members: &mut HashSet<Intersection>,
        group_color: Color,
        intersection: Intersection,
        group_number: i32,
    ) -> Result<bool, Error> {
        match group_assignments.get(intersection.row as usize, intersection.col as usize) {
            Some(&GroupPoint::Ungrouped) => {
                match self.points.get(intersection.row as usize, intersection.col as usize) {
                    Some(&PointState::Empty) => {
                        group_liberties.insert(intersection);
                        Ok(false)
                    }
                    Some(&PointState::Filled {
                        move_number: _,
                        stone_color,
                    }) => {
                        if stone_color == group_color {
                            group_assignments
                                .set(
                                    intersection.row as usize,
                                    intersection.col as usize,
                                    GroupPoint::Grouped { group_number },
                                )
                                .or(Err(Error::InvalidBoardNumber("Could not set group".to_string())))
                                .ok();
                            group_members.insert(intersection);
                            if intersection.row > 0 {
                                self.check_point(
                                    group_assignments,
                                    group_liberties,
                                    group_members,
                                    group_color,
                                    intersection.down(),
                                    group_number,
                                )?;
                            }
                            if intersection.row < self.size {
                                self.check_point(
                                    group_assignments,
                                    group_liberties,
                                    group_members,
                                    group_color,
                                    intersection.up(),
                                    group_number,
                                )?;
                            }
                            if intersection.col > 0 {
                                self.check_point(
                                    group_assignments,
                                    group_liberties,
                                    group_members,
                                    group_color,
                                    intersection.left(),
                                    group_number,
                                )?;
                            }
                            if intersection.col < self.size {
                                self.check_point(
                                    group_assignments,
                                    group_liberties,
                                    group_members,
                                    group_color,
                                    intersection.right(),
                                    group_number,
                                )?;
                            }
                            Ok(true)
                        } else {
                            Ok(false)
                        }
                    }
                    None => Err(Error::InvalidBoardNumber("Invalid board".to_string())),
                }
            }
            _ => Ok(false),
        }
    }

    pub fn to_ascii(&self) -> String {
        let mut ret: String = "".to_string();

        for r in 0..self.size {
            for c in 0..self.size {
                match self.points.get(r as usize, c as usize) {
                    Some(&PointState::Filled {
                        move_number: _,
                        stone_color,
                    }) => match stone_color {
                        Color::Black => {
                            ret += &"X  ".to_string();
                        }
                        Color::White => {
                            ret += &"O  ".to_string();
                        }
                    },
                    _ => {
                        ret += &".  ".to_string();
                    }
                }
            }
            ret += &"\n".to_string();
        }
        return ret;
    }

    //    pub fn groups_to_ascii(&self, group_assignments: &array2d::Array2D<GroupPoint>) -> String {
    //        let mut ret : String = "".to_string();
    //        for c in 0..self.size {
    //            for r in 0..self.size {
    //                match group_assignments.get(r, c) {
    //                    Some(&GroupPoint::Grouped { group_number }) => {
    //                        ret += &format!("{:<3}", group_number);
    //                    }
    //                    _ => {
    //                        ret += &".  ".to_string();
    //                    }
    //                }
    //            }
    //            ret += &"\n".to_string();
    //        }
    //        return ret;
    //    }
}
