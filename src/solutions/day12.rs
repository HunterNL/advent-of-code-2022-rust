use std::{
    cell::Cell,
    collections::{BinaryHeap, HashMap, HashSet},
    io,
};

use crate::{grid::Grid, vec2d::Vec2D};

use super::{DayOutput, LogicError, PartResult};

const START_MARKER: u8 = b'S';
const END_MARKER: u8 = b'E';

const VISUALIZE_PART_1: bool = false;
const INTERACTIVE_PART_2: bool = false;

fn retrace_path(mut closed_set: HashMap<Vec2D<i32>, Node>, last_node: &Node) -> Vec<Vec2D<i32>> {
    let mut path = vec![];
    let mut last_node = last_node.clone();
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
    }
}

fn fix_marker_elevations(n: &u8) -> u8 {
    match n {
        b'S' => b'a',
        b'E' => b'z',
        _ => *n,
    }
}

// Find path from marker E to any 'a' using bfs
fn find_path_down(map: &Grid<u8>) -> usize {
    let mut frontier: BinaryHeap<BFSNode> = BinaryHeap::new();
    let mut closed_set: HashMap<Vec2D<i32>, BFSNode> = HashMap::new();

    let start_pos = find_unique_character_index(map, END_MARKER)
        .map(|index| {
            map.position_of_index(index)
                .expect("Should find start marker index")
        })
        .expect("Should find start marker position");

    let start_node = BFSNode {
        pos: start_pos,
        cost_so_far: 0,
        parent: None,
    };

    frontier.push(start_node);

    while let Some(node) = frontier.pop() {
        // println!("Frontier size {}", frontier.len());
        let current_postion = node.pos;
        let current_elevation = map
            .get_by_vec(&current_postion)
            .map(fix_marker_elevations)
            .expect("Position should be on grid");

        if current_elevation == b'a' {
            return node.cost_so_far;
        }

        if INTERACTIVE_PART_2 {
            print_with_coloring_p2(map, &frontier, &closed_set, &current_postion);
            let _ = io::stdin().read_line(&mut String::new());
        }

        let mut neighbours: Vec<Vec2D<i32>> = Vec::new();

        map.get_neighbours(node.pos, &mut neighbours);

        // We can now only __decent__ once
        neighbours.retain(|neighbour_position| {
            let new_elevation = map
                .get_by_vec(neighbour_position)
                .map(fix_marker_elevations) // Replace S and E with a and z
                .unwrap();

            // Never allow a step that is too steep
            let too_steep = new_elevation < current_elevation - 1;
            !too_steep
        });

        neighbours.iter().for_each(|neighbour_position| {
            let movement_cost = 1;

            // If already in closed set, ignore
            if closed_set.contains_key(neighbour_position) {
                return;
            }

            // If already in frontier, ignore
            if frontier.iter().any(|node| node.pos == *neighbour_position) {
                return;
            }

            frontier.push(BFSNode {
                pos: *neighbour_position,
                cost_so_far: node.cost_so_far + movement_cost,
                parent: Some(current_postion),
            });
        });

        neighbours.clear();

        closed_set.insert(current_postion, node);
    }

    panic!("No path found");
}

fn print_with_coloring_p2(
    grid: &Grid<u8>,
    frontier: &BinaryHeap<BFSNode>,
    closed_set: &HashMap<Vec2D<i32>, BFSNode>,
    active_node: &Vec2D<i32>,
) {
    let mut frontier_positions = HashSet::new();
    let mut closed_positions = HashSet::new();

    for v in frontier {
        frontier_positions.insert(v.pos);
    }

    for v in closed_set {
        closed_positions.insert(v.0);
    }

    grid.iter_with_pos().for_each(|(pos, b)| {
        if pos.x == 0 {
            println!();
        }
        if (pos
            == Vec2D {
                x: active_node.x as usize,
                y: active_node.y as usize,
            })
        {
            // ACtive node
            print!("\x1b[33m"); // yellow
            print!("{}", *b as char);
            print!("\x1b[0m");
        } else if frontier_positions.contains({
            &Vec2D {
                x: pos.x as i32,
                y: pos.y as i32,
            }
        }) {
            // in frontier
            print!("\x1b[32m");
            print!("{}", *b as char);
            print!("\x1b[0m");
        } else if closed_positions.contains({
            &Vec2D {
                x: pos.x as i32,
                y: pos.y as i32,
            }
        }) {
            // in frontier
            print!("\x1b[31m");
            print!("{}", *b as char);
            print!("\x1b[0m"); // IN closed
        } else {
            // Not on path
            {
                print!("{}", *b as char);
            };
        }
    });
}

// Find path from marker S to marker E using a*
fn find_path(map: &Grid<u8>) -> Vec<Vec2D<i32>> {
    let mut frontier: BinaryHeap<Node> = BinaryHeap::new();
    let mut closed_set: HashMap<Vec2D<i32>, Node> = HashMap::new();

    let start_pos = find_unique_character_index(map, START_MARKER)
        .map(|index| {
            map.position_of_index(index)
                .expect("Should find start marker index")
        })
        .expect("Should find start marker position");

    let end_pos = find_unique_character_index(map, END_MARKER)
        .map(|index| {
            map.position_of_index(index)
                .expect("Should find end marker index")
        })
        .expect("Should find end marker position");

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
            return retrace_path(closed_set, &node);
        }

        // println!("Frontier size: {}", frontier.len());

        let current_position = node.pos;
        let current_elevation = map
            .get_by_vec(&current_position)
            .map(fix_marker_elevations) // Fix start marker elevation
            .expect("Valid position");

        let current_cost = node.cost_so_far.get();
        // let current_score = node.total_score.get();

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
                if closed_set_entry.total_score.get() <= neighbour_score as i32 {
                    // If the closed set contains a node with a lower or equal score we can disregard the current neighbor, a better path already exists
                    return;
                }
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
                });
            }
        });

        closed_set.insert(node.pos, node);

        neighbours.clear();
    }

    panic!("Pathfinding failed")
}

fn find_unique_character_index(map: &Grid<u8>, marker: u8) -> Option<usize> {
    map.iter().position(|b| *b == marker)
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

#[derive(PartialEq, Eq, Hash)]
struct BFSNode {
    pos: Vec2D<i32>,
    cost_so_far: usize,
    parent: Option<Vec2D<i32>>,
}

impl PartialOrd for BFSNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BFSNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cost_so_far.cmp(&other.cost_so_far).reverse()
    }
}

// https://adventofcode.com/2022/day/12
pub fn solve(input: &str) -> Result<DayOutput, LogicError> {
    let grid = Grid::from_str(input);
    let p1_movements = find_path(&grid);
    let p2_len = find_path_down(&grid);

    if VISUALIZE_PART_1 {
        print_with_coloring(&grid, &p1_movements);
    }

    Ok(DayOutput {
        part1: Some(PartResult::Int(p1_movements.len() as i32)),
        part2: Some(PartResult::Int(p2_len as i32)),
    })
}

fn print_with_coloring(grid: &Grid<u8>, path: &[Vec2D<i32>]) {
    let mut path_positions = HashSet::new();
    for v in path {
        path_positions.insert(*v);
    }

    grid.iter_with_pos().for_each(|(pos, b)| {
        if pos.x == 0 {
            println!();
        }
        if *b == b'a' {
            print!("\x1b[2m");
            print!("{}", *b as char);
            print!("\x1b[0m");
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
                print!("{}", *b as char);
            };
        }
    });
}

#[cfg(test)]
mod tests {

    use crate::{grid::Grid, solutions::day12::print_with_coloring};

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

        let grid = Grid::from_str(str);
        let movements = find_path(&grid);

        print_with_coloring(&grid, &movements);

        assert_eq!(movements.len(), 31);
    }
}
