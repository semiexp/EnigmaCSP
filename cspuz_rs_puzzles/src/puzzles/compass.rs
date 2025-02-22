use super::util;
use crate::graph;
use crate::serializer::{
    problem_to_url, url_to_problem, Choice, Combinator, Dict, Grid, HexInt, Map, Optionalize, Seq,
    Spaces,
};
use crate::solver::Solver;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompassClue {
    pub up: Option<i32>,
    pub down: Option<i32>,
    pub left: Option<i32>,
    pub right: Option<i32>,
}

pub fn solve_compass(
    clues: &[Vec<Option<CompassClue>>],
) -> Option<graph::BoolInnerGridEdgesIrrefutableFacts> {
    let (h, w) = util::infer_shape(clues);

    let mut solver = Solver::new();
    let edges = &graph::BoolInnerGridEdges::new(&mut solver, (h, w));
    solver.add_answer_key_bool(&edges.horizontal);
    solver.add_answer_key_bool(&edges.vertical);

    let mut compasses = vec![];
    for y in 0..h {
        for x in 0..w {
            if let Some(c) = clues[y][x] {
                compasses.push((y, x, c));
            }
        }
    }
    let group_id = solver.int_var_2d((h, w), 0, compasses.len() as i32 - 1);
    solver.add_expr(
        edges.horizontal.iff(
            group_id
                .slice((..(h - 1), ..))
                .ne(group_id.slice((1.., ..))),
        ),
    );
    solver.add_expr(
        edges.vertical.iff(
            group_id
                .slice((.., ..(w - 1)))
                .ne(group_id.slice((.., 1..))),
        ),
    );
    for (i, &(y, x, c)) in compasses.iter().enumerate() {
        graph::active_vertices_connected_2d(&mut solver, group_id.eq(i as i32));
        solver.add_expr(group_id.at((y, x)).eq(i as i32));
        if let Some(n) = c.up {
            solver.add_expr(group_id.slice((..y, ..)).eq(i as i32).count_true().eq(n));
        }
        if let Some(n) = c.down {
            solver.add_expr(
                group_id
                    .slice(((y + 1).., ..))
                    .eq(i as i32)
                    .count_true()
                    .eq(n),
            );
        }
        if let Some(n) = c.left {
            solver.add_expr(group_id.slice((.., ..x)).eq(i as i32).count_true().eq(n));
        }
        if let Some(n) = c.right {
            solver.add_expr(
                group_id
                    .slice((.., (x + 1)..))
                    .eq(i as i32)
                    .count_true()
                    .eq(n),
            );
        }
    }

    solver.irrefutable_facts().map(|f| f.get(edges))
}

type Problem = Vec<Vec<Option<CompassClue>>>;

fn combinator() -> impl Combinator<Problem> {
    Grid::new(Choice::new(vec![
        Box::new(Optionalize::new(Map::new(
            Seq::new(
                Choice::new(vec![
                    Box::new(Optionalize::new(HexInt)),
                    Box::new(Dict::new(None, ".")),
                ]),
                4,
            ),
            |c: CompassClue| Some(vec![c.up, c.down, c.left, c.right]),
            |c| {
                Some(CompassClue {
                    up: c[0],
                    down: c[1],
                    left: c[2],
                    right: c[3],
                })
            },
        ))),
        Box::new(Spaces::new(None, 'g')),
    ]))
}

pub fn serialize_problem(problem: &Problem) -> Option<String> {
    problem_to_url(combinator(), "compass", problem.clone())
}

pub fn deserialize_problem(url: &str) -> Option<Problem> {
    url_to_problem(combinator(), &["compass"], url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problem_for_tests() -> Problem {
        // https://puzz.link/p?compass/5/5/m..1.i25.1g53..i1..1m
        let mut problem: Vec<Vec<Option<CompassClue>>> = vec![vec![None; 5]; 5];
        problem[1][2] = Some(CompassClue {
            up: None,
            down: None,
            left: Some(1),
            right: None,
        });
        problem[2][1] = Some(CompassClue {
            up: Some(2),
            down: Some(5),
            left: None,
            right: Some(1),
        });
        problem[2][3] = Some(CompassClue {
            up: Some(5),
            down: Some(3),
            left: None,
            right: None,
        });
        problem[3][2] = Some(CompassClue {
            up: Some(1),
            down: None,
            left: None,
            right: Some(1),
        });
        problem
    }

    #[test]
    #[rustfmt::skip]
    fn test_compass_problem() {
        let problem = problem_for_tests();

        let ans = solve_compass(&problem);
        assert!(ans.is_some());
        let ans = ans.unwrap();

        let expected = graph::BoolInnerGridEdgesIrrefutableFacts {
            horizontal: crate::puzzle::util::tests::to_option_bool_2d([
                [0, 1, 1, 1, 0],
                [0, 1, 1, 1, 0],
                [0, 0, 0, 1, 0],
                [0, 0, 1, 1, 0],
            ]),
            vertical: crate::puzzle::util::tests::to_option_bool_2d([
                [1, 0, 0, 0],
                [1, 0, 0, 1],
                [0, 1, 1, 0],
                [0, 1, 0, 1],
                [0, 0, 1, 0],
            ]),
        };
        assert_eq!(ans, expected);
    }

    #[test]
    fn test_compass_serializer() {
        let problem = problem_for_tests();
        let url = "https://puzz.link/p?compass/5/5/m..1.i25.1g53..i1..1m";
        util::tests::serializer_test(problem, url, serialize_problem, deserialize_problem);
    }
}
