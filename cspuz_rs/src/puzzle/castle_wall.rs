use super::util;
use crate::graph;
use crate::solver::{Solver, FALSE};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Unspecified,
    Inside,
    Outside,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arrow {
    Unspecified(i32),
    Up(i32),
    Down(i32),
    Left(i32),
    Right(i32),
}

pub fn solve_castle_wall(
    clues: &[Vec<Option<(Side, Arrow)>>],
) -> Option<graph::BoolGridEdgesIrrefutableFacts> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let is_line = &graph::BoolGridEdges::new(&mut solver, (h - 1, w - 1));
    solver.add_answer_key_bool(&is_line.horizontal);
    solver.add_answer_key_bool(&is_line.vertical);
    graph::single_cycle_grid_edges(&mut solver, &is_line);

    let cell_sides = &solver.bool_var_2d((h - 1, w - 1));
    for y in 0..h {
        for x in 0..w {
            if y < h - 1 {
                let a = if x == 0 {
                    FALSE
                } else {
                    cell_sides.at((y, x - 1)).expr()
                };
                let b = if x == w - 1 {
                    FALSE
                } else {
                    cell_sides.at((y, x)).expr()
                };
                solver.add_expr(is_line.vertical.at((y, x)) ^ a.iff(b));
            }
            if x < w - 1 {
                let a = if y == 0 {
                    FALSE
                } else {
                    cell_sides.at((y - 1, x)).expr()
                };
                let b = if y == h - 1 {
                    FALSE
                } else {
                    cell_sides.at((y, x)).expr()
                };
                solver.add_expr(is_line.horizontal.at((y, x)) ^ a.iff(b));
            }
        }
    }

    for y in 0..h {
        for x in 0..w {
            if let Some((side, arrow)) = clues[y][x] {
                solver.add_expr(!(is_line.vertex_neighbors((y, x)).any()));
                match side {
                    Side::Unspecified => (),
                    Side::Inside => {
                        if y > 0 && x > 0 {
                            solver.add_expr(cell_sides.at((y - 1, x - 1)));
                        } else {
                            return None;
                        }
                    }
                    Side::Outside => {
                        if y > 0 && x > 0 {
                            solver.add_expr(!cell_sides.at((y - 1, x - 1)));
                        }
                    }
                }
                match arrow {
                    Arrow::Unspecified(_) => (),
                    Arrow::Up(n) => {
                        if n >= 0 {
                            solver.add_expr(
                                is_line.vertical.slice_fixed_x((..y, x)).count_true().eq(n),
                            );
                        }
                    }
                    Arrow::Down(n) => {
                        if n >= 0 {
                            solver.add_expr(
                                is_line.vertical.slice_fixed_x((y.., x)).count_true().eq(n),
                            );
                        }
                    }
                    Arrow::Left(n) => {
                        if n >= 0 {
                            solver.add_expr(
                                is_line
                                    .horizontal
                                    .slice_fixed_y((y, ..x))
                                    .count_true()
                                    .eq(n),
                            );
                        }
                    }
                    Arrow::Right(n) => {
                        if n >= 0 {
                            solver.add_expr(
                                is_line
                                    .horizontal
                                    .slice_fixed_y((y, x..))
                                    .count_true()
                                    .eq(n),
                            );
                        }
                    }
                }
            }
        }
    }

    solver.irrefutable_facts().map(|f| f.get(is_line))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem_for_tests() -> Vec<Vec<Option<(Side, Arrow)>>> {
        // https://puzsq.jp/main/puzzle_play.php?pid=7711
        let height = 10;
        let width = 10;
        let mut ret = vec![vec![None; width]; height];
        ret[0][0] = Some((Side::Unspecified, Arrow::Down(3)));
        ret[0][3] = Some((Side::Unspecified, Arrow::Down(2)));
        ret[0][6] = Some((Side::Unspecified, Arrow::Down(3)));
        ret[2][9] = Some((Side::Outside, Arrow::Down(4)));
        ret[3][3] = Some((Side::Unspecified, Arrow::Left(2)));
        ret[4][0] = Some((Side::Unspecified, Arrow::Right(4)));
        ret[5][7] = Some((Side::Inside, Arrow::Up(3)));
        ret[6][1] = Some((Side::Unspecified, Arrow::Right(4)));
        ret[6][4] = Some((Side::Unspecified, Arrow::Up(4)));
        ret[8][8] = Some((Side::Outside, Arrow::Up(4)));
        ret[9][1] = Some((Side::Unspecified, Arrow::Up(4)));
        ret[9][4] = Some((Side::Unspecified, Arrow::Up(4)));
        ret
    }

    #[test]
    fn test_castle_wall_problem() {
        let problem = problem_for_tests();
        let ans = solve_castle_wall(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();
        assert_eq!(ans.horizontal[4][7], Some(true));
        assert_eq!(ans.horizontal[4][8], Some(false));
        assert_eq!(ans.vertical[3][8], Some(true));
    }
}
