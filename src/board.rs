use alphabet::*;
use anyhow::Result;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;

use super::Color;
use super::GoError;
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
    size: usize,
    points: array2d::Array2D<PointState>,
}

impl Board {
    pub fn new(size: usize) -> Board {
        Board {
            size,
            points: array2d::Array2D::filled_with(PointState::Empty, size, size),
        }
    }

    pub fn get_point(&self, r: usize, c: usize) -> Result<PointState, GoError> {
        if let Some(p) = self.points.get(r, c).clone() {
            Ok(p.clone())
        } else {
            Err(GoError::InvalidMove("No Point".to_string()))
        }
    }

    pub fn render_diagram(
        &self,
        from_move: Option<usize>,
        all_moves: &BTreeMap<usize, Move>,
    ) -> Result<String, GoError> {
        let mut ret: String = "".to_string();
        let mut cap_ret: String = "".to_string();
        let mut max_move: usize = 0;
        let mut numbered_moves: HashSet<usize> = HashSet::new();
        let mut captured_moves: BTreeMap<Intersection, BTreeSet<usize>> = BTreeMap::new();

        for c in 0..self.size {
            for r in 0..self.size {
                match self.points.get(r, c) {
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
                        ret += &stone_color.to_string();
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
                if let Some(m) = &all_moves.get(&move_num) {
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
            }) = self.points.get(intersection.row, intersection.col)
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

    pub fn place_stone(
        &mut self,
        move_number: Option<usize>,
        intersection: Intersection,
        stone_color: Color,
    ) -> Result<(), GoError> {
        match self.points.get(intersection.row, intersection.col) {
            Some(&PointState::Empty) => {
                self.points
                    .set(
                        intersection.row,
                        intersection.col,
                        PointState::Filled {
                            move_number: move_number.unwrap_or(0),
                            stone_color,
                        },
                    )
                    .ok();
                if move_number.is_some() {
                    self.remove_captures(stone_color).ok();
                }
                Ok(())
            }
            Some(&PointState::Filled {
                move_number: _,
                stone_color: _,
            }) => Err(GoError::InvalidMove("Point already filled".to_string())),
            None => Err(GoError::InvalidMove("Invalid board".to_string())),
        }
    }

    fn remove_captures(&mut self, placed_stone_color: Color) -> Result<(), String> {
        let mut group_assignments: array2d::Array2D<GroupPoint> =
            array2d::Array2D::filled_with(GroupPoint::Ungrouped, self.size, self.size);
        let mut group_liberties: HashSet<Intersection> = HashSet::new();
        let mut group_members: HashSet<Intersection> = HashSet::new();
        let mut group_number = 1;
        for r in 0..self.size {
            for c in 0..self.size {
                match self.points.get(r, c) {
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
                                                intersection.row,
                                                intersection.col,
                                                PointState::Empty,
                                            )
                                            .or(Err(GoError::InvalidMove(
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
    ) -> Result<bool, GoError> {
        match group_assignments.get(intersection.row, intersection.col) {
            Some(&GroupPoint::Ungrouped) => {
                match self.points.get(intersection.row, intersection.col) {
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
                                    intersection.row,
                                    intersection.col,
                                    GroupPoint::Grouped { group_number },
                                )
                                .or(Err(GoError::InvalidMove("Could not set group".to_string())))
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
                    None => Err(GoError::InvalidMove("Invalid board".to_string())),
                }
            }
            _ => Ok(false),
        }
    }

    pub fn to_ascii(&self) -> String {
        let mut ret: String = "".to_string();

        for r in 0..self.size {
            for c in 0..self.size {
                match self.points.get(r, c) {
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
