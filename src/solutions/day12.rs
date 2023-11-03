use std::{
    borrow::BorrowMut,
    cell::Cell,
    collections::{BinaryHeap, HashMap, HashSet},
    f32::consts::E,
};

use crate::{grid::Grid, vec2d::Vec2D};

use super::{DayOutput, LogicError, PartResult};

const START_MARKER: u8 = b'S';
const END_MARKER: u8 = b'E';

fn find_node_neighbours(grid: &Grid<u8>, node: &Node, neighbours: &mut Vec<Node>) {}

// fn _() {}

fn retrace_path(mut closed_set: HashMap<Vec2D<i32>, Node>, last_node: Node) -> Vec<Vec2D<i32>> {
    let mut path = vec![];
    let mut last_node = last_node;
    loop {
        let parent_pos = last_node.parent.get();
        if let Some(parent_pos) = parent_pos {
            // println!("Retracing from {:?} to {:?}", last_node.pos, parent_pos);
            path.push(last_node.pos);
            last_node = closed_set
                .remove(&parent_pos)
                .expect("Closed set shoudl contain parent");
        } else {
            return path;
        }

        // let parent_node = closed_set.remove(&last_node.parent.unwrap());
    }
}

fn fix_marker_elevations(n: &u8) -> u8 {
    match n {
        b'S' => b'a',
        b'E' => b'z',
        _ => *n,
    }
}

fn find_path(map: &Grid<u8>) -> Vec<Vec2D<i32>> {
    let mut frontier: BinaryHeap<Node> = BinaryHeap::new();
    let mut closed_set: HashMap<Vec2D<i32>, Node> = HashMap::new();

    let start_index = map
        .iter()
        .position(|b| *b == START_MARKER)
        .expect("To find S in map");
    let start_pos = map
        .position_of_index(start_index)
        .expect("Index to be on the map");

    let end_index = map
        .iter()
        .position(|b| *b == END_MARKER)
        .expect("To find E in map");
    let end_pos = map
        .position_of_index(end_index)
        .expect("Index to be on the map");

    let hueristic = |position: &Vec2D<i32>| position.distance_manhatten(&end_pos);

    // let start_node =  create_node_for_position(pos, end_pos, parent: &node);
    let start_node = Node {
        total_score: Cell::new(hueristic(&start_pos)),
        pos: start_pos,
        cost_so_far: Cell::new(0),
        hueristic_score: hueristic(&start_pos),
        parent: Cell::new(None),
    };

    frontier.push(start_node);

    let mut neighbours: Vec<Vec2D<i32>> = Vec::new();

    while let Some(node) = frontier.pop() {
        if node.pos == end_pos {
            print!("Found!");
            return retrace_path(closed_set, node);
        }

        println!("Frontier size: {}", frontier.len());

        let current_position = node.pos;
        let current_elevation = map
            .get_by_vec(&current_position)
            .map(fix_marker_elevations) // Fix start marker elevation
            .expect("Valid position");

        let current_cost = node.cost_so_far.get();
        let current_score = node.total_score.get();

        map.get_neighbours(node.pos, &mut neighbours);

        // Filter-in-place to only accessible neighbors, no climbing gear!
        // New position can only be 1 higher
        neighbours.retain(|neighbour_position| {
            let new_elevation = map
                .get_by_vec(neighbour_position)
                .map(fix_marker_elevations) // Replace S and E with a and z
                .unwrap();

            // Never allow a step that is too steep
            let too_steep = new_elevation > current_elevation + 1;
            !too_steep
        });

        neighbours.iter().for_each(|neighbour_position| {
            let movementcost = 1; // Cost to move to a neighbour is always 1
            let h = hueristic(neighbour_position);
            let neighbour_score = current_cost + movementcost + h as usize;

            // If the entry is in the closed set
            if let Some(closed_set_entry) = closed_set.get(neighbour_position) {
                if closed_set_entry.total_score.get() < neighbour_score as i32 {
                    // If the closed set contains a node with a lower score we can disregard the current neighbor, a better path already exists
                    return;
                }

                // If we can shorten the path, throw it back onto the frontier
                // if neighbour_score < closed_set_entry.total_score.get() as usize {
                //     let closed_set_entry = closed_set.remove(neighbour_position).unwrap();
                //     closed_set_entry
                //         .cost_so_far
                //         .set(current_cost + movementcost);
                //     closed_set_entry.total_score.set(neighbour_score as i32);
                //     closed_set_entry.parent.set(Some(*neighbour_position));

                //     frontier.push(closed_set_entry)
                // }
            }

            // Possible existing entry in the frontier
            let node_option_in_frontier =
                frontier.iter().find(|node| node.pos == *neighbour_position);

            if let Some(frontier_node) = node_option_in_frontier {
                // There's a shorter path via our current node, apply it
                if neighbour_score < frontier_node.total_score.get() as usize {
                    frontier_node.total_score.set(neighbour_score as i32);
                    frontier_node.parent.set(Some(current_position));
                    frontier_node.cost_so_far.set(current_cost + movementcost);
                }
                // Else just ignore
            } else {
                frontier.push(Node {
                    pos: *neighbour_position,
                    cost_so_far: Cell::new(current_cost + movementcost),
                    hueristic_score: h,
                    parent: Cell::new(Some(current_position)),
                    total_score: Cell::new(neighbour_score as i32),
                })
            }

            //     if neighbour_score < closed_set_entry.total_score.get() as usize {
            //         println!(
            //             "Found shorter path for {:?} in closed set, setting parent to {:?}",
            //             closed_set_entry.pos, current_position
            //         );
            //         closed_set_entry.parent.set(Some(current_position));
            //         closed_set_entry
            //             .cost_so_far
            //             .set(current_cost + movementcost)
            //             closed_set_entry.s
            //     }
            //     return;
            // }

            // println!("Adding {:?} to frontier", neighbour_position);
        });

        closed_set.insert(node.pos, node);

        neighbours.clear();
    }

    panic!("Pathfinding failed")
}

fn create_node_for_position(pos: Vec2D<i32>, target_position: Vec2D<i32>) -> Node {
    !todo!();
    // Node{ pos, accumelated: todo!(), hueristic_score: todo!()
}

#[derive(PartialEq, Eq, Clone)]
struct Node {
    pos: Vec2D<i32>,
    cost_so_far: Cell<usize>,
    hueristic_score: i32,
    total_score: Cell<i32>,
    parent: Cell<Option<Vec2D<i32>>>,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.total_score
            .cmp(&other.total_score)
            // .then(self.hueristic_score.cmp(&other.hueristic_score))
            .reverse()
    }
}

fn print_path_on_grid(path: &[Vec2D<i32>], map: &mut Grid<u8>) {
    path.iter().for_each(|position| map.set(position, b'*'));
}

// fn carve_path_on_grid(path: &[Vec2D<i32>], map: &mut Grid<u8>) {
//     map.iter_mut(|b|)

//     path.iter().for_each(|position| map.set(position, b'*'));
// }

// https://adventofcode.com/2022/day/12
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let mut grid = Grid::from_str(input);
    let movements = find_path(&grid);

    // println!("{}", grid);

    print_with_coloring(&mut grid, &movements);

    // println!("{}", grid);

    Ok(DayOutput {
        part1: Some(PartResult::Int(movements.len() as i32)),
        part2: None,
    })
}

fn filter_map_to_path(grid: &mut Grid<u8>, path: &Vec<Vec2D<i32>>) {
    let mut path_positions = HashSet::new();
    path.iter().for_each(|v| {
        path_positions.insert(*v);
    });

    grid.iter_mut_with_pos().for_each(|(pos, b)| {
        if !path_positions.contains({
            &Vec2D {
                x: pos.x as i32,
                y: pos.y as i32,
            }
        }) {
            *b = b' ';
        }
    });
}

fn print_with_coloring(grid: &mut Grid<u8>, path: &Vec<Vec2D<i32>>) {
    let mut path_positions = HashSet::new();
    path.iter().for_each(|v| {
        path_positions.insert(*v);
    });

    grid.iter_with_pos().for_each(|(pos, b)| {
        if pos.x == 0 {
            println!();
        }
        if (*b == b'a') {
            print!("{}", b' ' as char);
            return;
        }
        if path_positions.contains({
            &Vec2D {
                x: pos.x as i32,
                y: pos.y as i32,
            }
        }) {
            // On path
            print!("\x1b[32m");
            print!("{}", *b as char);
            print!("\x1b[0m");
        } else {
            // Not on path
            {
                print!("{}", *b as char)
            };
        }
    });
}

#[cfg(test)]
mod tests {
    use crate::{
        grid::{self, Grid},
        solutions::day12::{filter_map_to_path, print_path_on_grid},
    };

    use super::find_path;

    #[test]
    fn day() -> Result<(), String> {
        super::super::tests::test_day(12, super::solve)
    }

    #[test]
    fn example() {
        let str = "Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

        let mut grid = Grid::from_str(str);
        let movements = find_path(&grid);

        println!("{}", grid);

        filter_map_to_path(&mut grid, &movements);

        println!("{}", grid);

        assert_eq!(movements.len(), 31);
    }
}
